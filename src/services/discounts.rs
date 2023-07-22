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
    models::discount::{Discount, InsertDiscount, UpdateDiscount},
    services::pagination_params::PaginationParams,
    services::responses_dto::*,
    services::service_error::ServiceError,
    utils::{deserialization::MaybeAbsent, pagination::Paginable},
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_discounts)
        .service(fetch_discount)
        .service(create_discount)
        .service(update_discount_partially)
        .service(update_discount_completely)
        .service(delete_discount);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreateDiscountPayload {
    dealership_rif: String,
    discount_percentage: BigDecimal,
    required_annual_service_usage_count: i16,
}

#[post("/")]
async fn create_discount(
    Json(payload): Json<CreateDiscountPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_discount = InsertDiscount {
        dealership_rif: payload.dealership_rif,
        discount_percentage: payload.discount_percentage,
        required_annual_service_usage_count: payload.required_annual_service_usage_count,
    }
    .insert(db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidCreateError(
                "The specified dealershipRif does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to insert the discount into the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: created_discount,
    }))
}

#[get("/")]
async fn fetch_discounts(
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

        let fetched_discounts = fetch_discounts_paginated(per_page, page_no, db.get_ref()).await?;

        let total_discounts = Discount::count(db.get_ref())
            .await
            .context("Failed to count the discounts from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_discounts,
                pagination: Pagination::new(total_discounts, page_no, per_page),
            });

        return Ok(response);
    }

    let fetched_discounts = fetch_all_discounts(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_discounts,
        });

    Ok(response)
}

async fn fetch_all_discounts(db: &Pool<Postgres>) -> Result<Vec<Discount>, ServiceError> {
    let fetched_discounts = Discount::select_all(db)
        .await
        .context("Failed to fetch the discounts from the database")?;
    Ok(fetched_discounts)
}

async fn fetch_discounts_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<Discount>, ServiceError> {
    let fetched_discounts = Discount::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the discounts from the database for the provided page")?;

    Ok(fetched_discounts.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct DiscountManipulationParams {
    discount_number: i32,
    dealership_rif: String,
}

#[get("/view/")]
async fn fetch_discount(
    Query(params): Query<DiscountManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_discount =
        Discount::select(params.discount_number, params.dealership_rif, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("discount".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the discount from the database"),
                ),
            })?;

    Ok(Json(NonPaginatedResponseDto {
        data: fetched_discount,
    }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdateDiscountPartiallyPayload {
    dealership_rif: MaybeAbsent<String>,
    discount_percentage: MaybeAbsent<BigDecimal>,
    required_annual_service_usage_count: MaybeAbsent<i16>,
}

#[patch("/")]
async fn update_discount_partially(
    Query(params): Query<DiscountManipulationParams>,
    Json(payload): Json<UpdateDiscountPartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let dealership_to_update =
        Discount::select(params.discount_number, params.dealership_rif, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("discount".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err)
                        .context("Failed to fetch the discount to update from the database"),
                ),
            })?;

    let updated_discount = UpdateDiscount {
        dealership_rif: payload.dealership_rif.into(),
        discount_percentage: payload.discount_percentage.into(),
        required_annual_service_usage_count: payload.required_annual_service_usage_count.into(),
    }
    .update(dealership_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified dealershipRif does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the dealership from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_discount,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateDiscountCompletelyPayload {
    dealership_rif: String,
    discount_percentage: BigDecimal,
    required_annual_service_usage_count: i16,
}

#[put("/")]
async fn update_discount_completely(
    Query(params): Query<DiscountManipulationParams>,
    Json(payload): Json<UpdateDiscountCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let city_to_update =
        Discount::select(params.discount_number, params.dealership_rif, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("discount".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err)
                        .context("Failed to fetch the discount to update from the database"),
                ),
            })?;

    let updated_discount = UpdateDiscount {
        dealership_rif: Some(payload.dealership_rif),
        discount_percentage: Some(payload.discount_percentage),
        required_annual_service_usage_count: Some(payload.required_annual_service_usage_count),
    }
    .update(city_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified dealershipRif does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the discount from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_discount,
    }))
}

#[delete("/")]
async fn delete_discount(
    Query(params): Query<DiscountManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_discount =
        Discount::delete(params.discount_number, params.dealership_rif, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("discount".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err)
                        .context("Failed to fetch the discount to delete from the database"),
                ),
            })?;

    Ok(Json(NonPaginatedResponseDto {
        data: deleted_discount,
    }))
}
