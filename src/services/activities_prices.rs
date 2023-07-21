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
    models::activity_price::{ActivityPrice, InsertActivityPrice, UpdateActivityPrice},
    services::pagination_params::PaginationParams,
    services::responses_dto::*,
    services::service_error::ServiceError,
    utils::{deserialization::MaybeAbsent, pagination::Paginable},
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_activities_prices)
        .service(fetch_activity_price)
        .service(create_activity_price)
        .service(update_activity_price_partially)
        .service(update_activity_price_completely)
        .service(delete_activity_price);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreateActivityPricePayload {
    activity_number: i32,
    service_id: i32,
    dealership_rif: String,
    price_per_hour: BigDecimal,
}

#[post("/")]
async fn create_activity_price(
    Json(payload): Json<CreateActivityPricePayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_activity_price = InsertActivityPrice {
        activity_number: payload.activity_number,
        service_id: payload.service_id,
        dealership_rif: payload.dealership_rif,
        price_per_hour: payload.price_per_hour,
    }
    .insert(db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidCreateError(
                "One of the specified values for the following keys does not exist: activityNumber, serviceId, dealershipRif".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to insert the activity price into the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: created_activity_price,
    }))
}

#[get("/")]
async fn fetch_activities_prices(
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

        let fetched_activities_prices =
            fetch_activities_prices_paginated(per_page, page_no, db.get_ref()).await?;

        let total_activities_prices = ActivityPrice::count(db.get_ref())
            .await
            .context("Failed to count the activities prices from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_activities_prices,
                pagination: Pagination::new(total_activities_prices, page_no, per_page),
            });

        return Ok(response);
    }

    let fetched_activities_prices = fetch_all_activities_prices(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_activities_prices,
        });

    Ok(response)
}

async fn fetch_all_activities_prices(db: &Pool<Postgres>) -> Result<Vec<ActivityPrice>, ServiceError> {
    let fetched_activities_prices = ActivityPrice::select_all(db)
        .await
        .context("Failed to fetch the activities prices from the database")?;
    Ok(fetched_activities_prices)
}

async fn fetch_activities_prices_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<ActivityPrice>, ServiceError> {
    let fetched_activities_prices = ActivityPrice::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the activities prices from the database for the provided page")?;

    Ok(fetched_activities_prices.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct ActivityPriceManipulationParams {
    activity_number: i32,
    service_id: i32,
    dealership_rif: String
}

#[get("/view/")]
async fn fetch_activity_price(
    Query(params): Query<ActivityPriceManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_activity_price =
        ActivityPrice::select(params.activity_number, params.service_id, params.dealership_rif, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("activity price".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the activity price from the database"),
                ),
            })?;

    Ok(Json(NonPaginatedResponseDto {
        data: fetched_activity_price,
    }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdateActivityPricePartiallyPayload {
    activity_number: MaybeAbsent<i32>,
    service_id: MaybeAbsent<i32>,
    dealership_rif: MaybeAbsent<String>,
    price_per_hour: MaybeAbsent<BigDecimal>,
}

#[patch("/")]
async fn update_activity_price_partially(
    Query(params): Query<ActivityPriceManipulationParams>,
    Json(payload): Json<UpdateActivityPricePartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let activity_to_update =
        ActivityPrice::select(params.activity_number, params.service_id, params.dealership_rif, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("activity price".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err)
                        .context("Failed to fetch the activity to update from the database"),
                ),
            })?;

    let updated_activity_price = UpdateActivityPrice {
        activity_number: payload.activity_number.into(),
        service_id: payload.service_id.into(),
        dealership_rif: payload.dealership_rif.into(),
        price_per_hour: payload.price_per_hour.into(),
    }
    .update(activity_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "One of the specified values for the following keys does not exist: activityNumber, serviceId, dealershipRif".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the activity price from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_activity_price,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateActivityPriceCompletelyPayload {
    activity_number: i32,
    service_id: i32,
    dealership_rif: String,
    price_per_hour: BigDecimal,
}

#[put("/")]
async fn update_activity_price_completely(
    Query(params): Query<ActivityPriceManipulationParams>,
    Json(payload): Json<UpdateActivityPriceCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let activity_to_update =
        ActivityPrice::select(params.activity_number, params.service_id, params.dealership_rif, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("activity price".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err)
                        .context("Failed to fetch the activity price to update from the database"),
                ),
            })?;

    let updated_activity_price = UpdateActivityPrice {
        activity_number: Some(payload.activity_number),
        service_id: Some(payload.service_id),
        dealership_rif: Some(payload.dealership_rif),
        price_per_hour: Some(payload.price_per_hour),
    }
    .update(activity_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "One of the specified values for the following keys does not exist: activityNumber, serviceId, dealershipRif".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the activity price from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_activity_price,
    }))
}

#[delete("/")]
async fn delete_activity_price(
    Query(params): Query<ActivityPriceManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_activity_price =
        ActivityPrice::delete(params.activity_number, params.service_id, params.dealership_rif, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("activity price".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to get the activity price to delete from the database"),
                ),
            })?;

    Ok(Json(NonPaginatedResponseDto {
        data: deleted_activity_price,
    }))
}
