use actix_web::{
    delete, get,
    http::{header::ContentType, StatusCode},
    patch, post, put,
    web::{Data, Json, Query, ServiceConfig},
    HttpResponse, Responder,
};
use anyhow::{anyhow, Context};
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::{
    models::product::{InsertProduct, Product, UpdateProduct},
    services::pagination_params::PaginationParams,
    services::responses_dto::*,
    services::service_error::ServiceError,
    utils::{deserialization::MaybeAbsent, pagination::Paginable},
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_products)
        .service(fetch_product)
        .service(create_product)
        .service(update_product_partially)
        .service(update_product_completely)
        .service(delete_product);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreateProductPayload {
    name: String,
    description: String,
    is_ecologic: bool,
    supply_line_id: i32,
}

#[post("/")]
async fn create_product(
    Json(payload): Json<CreateProductPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_product = InsertProduct {
        name: payload.name,
        description: payload.description,
        is_ecologic: payload.is_ecologic,
        supply_line_id: payload.supply_line_id,
    }
    .insert(db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidCreateError(
                "The specified supplyLineId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to insert the product into the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: created_product,
    }))
}

#[get("/")]
async fn fetch_products(
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

        let fetched_products = fetch_products_paginated(per_page, page_no, db.get_ref()).await?;

        let total_products = Product::count(db.get_ref())
            .await
            .context("Failed to count the products from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_products,
                pagination: Pagination::new(total_products, page_no, per_page),
            });

        return Ok(response);
    }

    let fetched_products = fetch_all_products(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_products,
        });

    Ok(response)
}

async fn fetch_all_products(db: &Pool<Postgres>) -> Result<Vec<Product>, ServiceError> {
    let fetched_products = Product::select_all(db)
        .await
        .context("Failed to fetch the products from the database")?;
    Ok(fetched_products)
}

async fn fetch_products_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<Product>, ServiceError> {
    let fetched_products = Product::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the products from the database for the provided page")?;

    Ok(fetched_products.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct ProductManipulationParams {
    id: i32,
}

#[get("/view/")]
async fn fetch_product(
    Query(params): Query<ProductManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_product =
        Product::select(params.id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("product".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the product from the database"),
                ),
            })?;

    Ok(Json(NonPaginatedResponseDto {
        data: fetched_product,
    }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdateProductPartiallyPayload {
    name: MaybeAbsent<String>,
    description: MaybeAbsent<String>,
    is_ecologic: MaybeAbsent<bool>,
    supply_line_id: MaybeAbsent<i32>,
}

#[patch("/")]
async fn update_product_partially(
    Query(params): Query<ProductManipulationParams>,
    Json(payload): Json<UpdateProductPartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let city_to_update =
        Product::select(params.id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("product".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the product to update from the database"),
                ),
            })?;

    let updated_product = UpdateProduct {
        name: payload.name.into(),
        description: payload.description.into(),
        is_ecologic: payload.is_ecologic.into(),
        supply_line_id: payload.supply_line_id.into(),
    }
    .update(city_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified supplyLineId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the product from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_product,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateProductCompletelyPayload {
    name: String,
    description: String,
    is_ecologic: bool,
    supply_line_id: i32,
}

#[put("/")]
async fn update_product_completely(
    Query(params): Query<ProductManipulationParams>,
    Json(payload): Json<UpdateProductCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let city_to_update =
        Product::select(params.id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("product".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the product to update from the database"),
                ),
            })?;

    let updated_product = UpdateProduct {
        name: Some(payload.name),
        description: Some(payload.description),
        is_ecologic: Some(payload.is_ecologic),
        supply_line_id: Some(payload.supply_line_id),
    }
    .update(city_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified supplyLineId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the product from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_product,
    }))
}

#[delete("/")]
async fn delete_product(
    Query(params): Query<ProductManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_product =
        Product::delete(params.id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("product".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the product to delete from the database"),
                ),
            })?;

    Ok(Json(NonPaginatedResponseDto {
        data: deleted_product,
    }))
}
