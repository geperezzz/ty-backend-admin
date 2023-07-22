use actix_web::{
    delete, get,
    http::{header::ContentType, StatusCode},
    patch, post, put,
    web::{Data, Json, Query, ServiceConfig},
    HttpResponse, Responder,
};
use anyhow::{anyhow, Context};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::{
    models::stock_item::{InsertStockItem, StockItem, UpdateStockItem},
    services::pagination_params::PaginationParams,
    services::responses_dto::*,
    services::service_error::ServiceError,
    utils::{deserialization::MaybeAbsent, pagination::Paginable},
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_stock)
        .service(fetch_stock_item)
        .service(create_stock_item)
        .service(update_stock_item_partially)
        .service(update_stock_item_completely)
        .service(delete_stock_item);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreateStockItemPayload {
    product_id: i32,
    dealership_rif: String,
    product_cost: BigDecimal,
    product_count: i32,
    vendor_name: String,
    max_capacity: i32,
    min_capacity: i32,
}

#[post("/")]
async fn create_stock_item(
    Json(payload): Json<CreateStockItemPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_stock_item = InsertStockItem {
        product_id: payload.product_id,
        dealership_rif: payload.dealership_rif,
        product_cost: payload.product_cost,
        product_count: payload.product_count,
        vendor_name: payload.vendor_name,
        max_capacity: payload.max_capacity,
        min_capacity: payload.min_capacity,
    }
    .insert(db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ServiceError::InvalidCreateError(
                "Already exists a stock item with the specified productId and dealershipRif".to_string(),
                anyhow!(err),
            )
        },
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidCreateError(
                "One of the specified values for one of the following keys does not exist: productId, dealershipRif".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to insert the stock item into the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: created_stock_item,
    }))
}

#[get("/")]
async fn fetch_stock(
    Query(pagination_params): Query<PaginationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<HttpResponse, ServiceError> {
    if pagination_params.per_page.is_some() && pagination_params.page_no.is_none() {
        return Err(ServiceError::MissingQueryParamError(
            "Missing query param page-no".to_string(),
        ));
    }

    if pagination_params.per_page.is_none() && pagination_params.page_no.is_some() {
        return Err(ServiceError::MissingQueryParamError(
            "Missing query param per-page".to_string(),
        ));
    }

    if pagination_params.per_page.is_some() && pagination_params.page_no.is_some() {
        let (per_page, page_no) = (
            pagination_params.per_page.unwrap(),
            pagination_params.page_no.unwrap(),
        );

        if page_no <= 0 {
            return Err(ServiceError::InvalidQueryParamValueError(
                "Query param page-no must be greater than 0".to_string(),
            ));
        }

        if per_page <= 0 {
            return Err(ServiceError::InvalidQueryParamValueError(
                "Query param per-page must be greater than 0".to_string(),
            ));
        }

        let fetched_stock_items = fetch_stock_paginated(per_page, page_no, db.get_ref()).await?;

        let total_stock_items = StockItem::count(db.get_ref())
            .await
            .context("Failed to count the stock from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_stock_items,
                pagination: Pagination::new(total_stock_items, page_no, per_page),
            });

        return Ok(response);
    }

    let fetched_stock_items = fetch_all_stock(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_stock_items,
        });

    Ok(response)
}

async fn fetch_all_stock(db: &Pool<Postgres>) -> Result<Vec<StockItem>, ServiceError> {
    let fetched_stock_items = StockItem::select_all(db)
        .await
        .context("Failed to fetch the stock from the database")?;
    Ok(fetched_stock_items)
}

async fn fetch_stock_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<StockItem>, ServiceError> {
    let fetched_stock_items = StockItem::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the stock from the database for the provided page")?;

    Ok(fetched_stock_items.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct StockItemManipulationParams {
    product_id: i32,
    dealership_rif: String
}

#[get("/view/")]
async fn fetch_stock_item(
    Query(params): Query<StockItemManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_stock_item = StockItem::select(params.product_id, params.dealership_rif, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("stock item".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the stock item from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: fetched_stock_item,
    }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdateStockItemPartiallyPayload {
    product_id: MaybeAbsent<i32>,
    dealership_rif: MaybeAbsent<String>,
    product_cost: MaybeAbsent<BigDecimal>,
    product_count: MaybeAbsent<i32>,
    vendor_name: MaybeAbsent<String>,
    max_capacity: MaybeAbsent<i32>,
    min_capacity: MaybeAbsent<i32>,
}

#[patch("/")]
async fn update_stock_item_partially(
    Query(params): Query<StockItemManipulationParams>,
    Json(payload): Json<UpdateStockItemPartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let state_to_update =
        StockItem::select(params.product_id, params.dealership_rif, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("stock item".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the stock item to update from the database"),
                ),
            })?;

    let updated_stock_item = UpdateStockItem {
        product_id: payload.product_id.into(),
        dealership_rif: payload.dealership_rif.into(),
        product_cost: payload.product_cost.into(),
        product_count: payload.product_count.into(),
        vendor_name: payload.vendor_name.into(),
        max_capacity: payload.max_capacity.into(),
        min_capacity: payload.min_capacity.into(),
    }
    .update(state_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ServiceError::InvalidUpdateError(
                "Already exists a stock item with the specified activityNumber and serviceId".to_string(),
                anyhow!(err),
            )
        },
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "One of the specified values for one of the following keys does not exist: productId, dealershipRif".to_string(),
                anyhow!(err),
            )
        },
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the stock item from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_stock_item,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateStockItemCompletelyPayload {
    product_id: i32,
    dealership_rif: String,
    product_cost: BigDecimal,
    product_count: i32,
    vendor_name: String,
    max_capacity: i32,
    min_capacity: i32,
}

#[put("/")]
async fn update_stock_item_completely(
    Query(params): Query<StockItemManipulationParams>,
    Json(payload): Json<UpdateStockItemCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let state_to_update =
        StockItem::select(params.product_id, params.dealership_rif, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("stock item".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the stock item to update from the database"),
                ),
            })?;

    let updated_stock_item = UpdateStockItem {
        product_id: Some(payload.product_id),
        dealership_rif: Some(payload.dealership_rif),
        product_cost: Some(payload.product_cost),
        product_count: Some(payload.product_count),
        vendor_name: Some(payload.vendor_name),
        max_capacity: Some(payload.max_capacity),
        min_capacity: Some(payload.min_capacity),
    }
    .update(state_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ServiceError::InvalidUpdateError(
                "Already exists a stock item with the specified activityNumber and serviceId".to_string(),
                anyhow!(err),
            )
        },
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "One of the specified values for one of the following keys does not exist: productId, dealershipRif".to_string(),
                anyhow!(err),
            )
        },
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the stock item from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_stock_item,
    }))
}

#[delete("/")]
async fn delete_stock_item(
    Query(params): Query<StockItemManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_stock_item = StockItem::delete(params.product_id, params.dealership_rif, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("stock item".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to get the stock item to delete from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: deleted_stock_item,
    }))
}
