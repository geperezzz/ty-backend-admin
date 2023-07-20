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
    models::activity::{Activity, InsertActivity, UpdateActivity},
    services::pagination_params::PaginationParams,
    services::responses_dto::*,
    services::service_error::ServiceError,
    utils::{deserialization::MaybeAbsent, pagination::Paginable},
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_activities)
        .service(fetch_activity)
        .service(create_activity)
        .service(update_activity_partially)
        .service(update_activity_completely)
        .service(delete_activity);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreateActivityPayload {
    service_id: i32,
    description: String,
    price_per_hour: BigDecimal,
}

#[post("/")]
async fn create_activity(
    Json(payload): Json<CreateActivityPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_activity = InsertActivity {
        service_id: payload.service_id,
        description: payload.description,
        price_per_hour: payload.price_per_hour,
    }
    .insert(db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidCreateError(
                "The specified serviceId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to create the activity from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: created_activity,
    }))
}

#[get("/")]
async fn fetch_activities(
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

        let fetched_activities =
            fetch_activities_paginated(per_page, page_no, db.get_ref()).await?;

        let total_activities = Activity::count(db.get_ref())
            .await
            .context("Failed to count the activities from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_activities,
                pagination: Pagination::new(total_activities, page_no, per_page),
            });

        return Ok(response);
    }

    let fetched_activities = fetch_all_activities(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_activities,
        });

    Ok(response)
}

async fn fetch_all_activities(db: &Pool<Postgres>) -> Result<Vec<Activity>, ServiceError> {
    let fetched_activities = Activity::select_all(db)
        .await
        .context("Failed to fetch the activities from the database")?;
    Ok(fetched_activities)
}

async fn fetch_activities_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<Activity>, ServiceError> {
    let fetched_activities = Activity::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the activities from the database for the provided page")?;

    Ok(fetched_activities.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct ActivityManipulationParams {
    activity_number: i32,
    service_id: i32,
}

#[get("/view/")]
async fn fetch_activity(
    Query(params): Query<ActivityManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_activity =
        Activity::select(params.activity_number, params.service_id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("activity".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the activity from the database"),
                ),
            })?;

    Ok(Json(NonPaginatedResponseDto {
        data: fetched_activity,
    }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdateActivityPartiallyPayload {
    service_id: MaybeAbsent<i32>,
    description: MaybeAbsent<String>,
    price_per_hour: MaybeAbsent<BigDecimal>,
}

#[patch("/")]
async fn update_activity_partially(
    Query(params): Query<ActivityManipulationParams>,
    Json(payload): Json<UpdateActivityPartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let activity_to_update =
        Activity::select(params.activity_number, params.service_id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("activity".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err)
                        .context("Failed to fetch the activity to update from the database"),
                ),
            })?;

    let updated_activity = UpdateActivity {
        service_id: payload.service_id.into(),
        description: payload.description.into(),
        price_per_hour: payload.price_per_hour.into(),
    }
    .update(activity_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified serviceId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the activity from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_activity,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateActivityCompletelyPayload {
    service_id: i32,
    description: String,
    price_per_hour: BigDecimal,
}

#[put("/")]
async fn update_activity_completely(
    Query(params): Query<ActivityManipulationParams>,
    Json(payload): Json<UpdateActivityCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let activity_to_update =
        Activity::select(params.activity_number, params.service_id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("activity".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err)
                        .context("Failed to fetch the activity to update from the database"),
                ),
            })?;

    let updated_activity = UpdateActivity {
        service_id: Some(payload.service_id),
        description: Some(payload.description),
        price_per_hour: Some(payload.price_per_hour),
    }
    .update(activity_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified serviceId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the activity from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_activity,
    }))
}

#[delete("/")]
async fn delete_activity(
    Query(params): Query<ActivityManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_activity =
        Activity::delete(params.activity_number, params.service_id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("activity".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to get the activity to delete from the database"),
                ),
            })?;

    Ok(Json(NonPaginatedResponseDto {
        data: deleted_activity,
    }))
}
