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
    models::dealership::{Dealership, InsertDealership, UpdateDealership},
    services::pagination_params::PaginationParams,
    services::responses_dto::*,
    services::service_error::ServiceError,
    utils::{
        deserialization::{MaybeAbsent, MaybeNull},
        pagination::Paginable,
    },
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_dealerships)
        .service(fetch_dealership)
        .service(create_dealership)
        .service(update_dealership_partially)
        .service(update_dealership_completely)
        .service(delete_dealership);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreateDealershipPayload {
    rif: String,
    name: String,
    city_number: i32,
    state_id: i32,
    manager_national_id: MaybeNull<String>,
}

#[post("/")]
async fn create_dealership(
    Json(payload): Json<CreateDealershipPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_dealership = InsertDealership {
        rif: payload.rif,
        name: payload.name,
        city_number: payload.city_number,
        state_id: payload.state_id,
        manager_national_id: payload.manager_national_id.into(),
    }
    .insert(db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ServiceError::InvalidCreateError(
                "The specified rif already exists or the specified managerNationalId is already being used".to_string(), 
                anyhow!(err),
            )
        }
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidCreateError(
                "The specified cityNumber, stateId or managerNationalId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to insert the dealership into the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: created_dealership,
    }))
}

#[get("/")]
async fn fetch_dealerships(
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

        let fetched_dealerships =
            fetch_dealerships_paginated(per_page, page_no, db.get_ref()).await?;

        let total_dealerships = Dealership::count(db.get_ref())
            .await
            .context("Failed to count the dealerships from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_dealerships,
                pagination: Pagination::new(total_dealerships, page_no, per_page),
            });

        return Ok(response);
    }

    let fetched_dealerships = fetch_all_dealerships(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_dealerships,
        });

    Ok(response)
}

async fn fetch_all_dealerships(db: &Pool<Postgres>) -> Result<Vec<Dealership>, ServiceError> {
    let fetched_dealerships = Dealership::select_all(db)
        .await
        .context("Failed to fetch the dealerships from the database")?;
    Ok(fetched_dealerships)
}

async fn fetch_dealerships_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<Dealership>, ServiceError> {
    let fetched_dealerships = Dealership::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the dealerships from the database for the provided page")?;

    Ok(fetched_dealerships.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct DealershipManipulationParams {
    rif: String,
}

#[get("/view/")]
async fn fetch_dealership(
    Query(params): Query<DealershipManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_dealership = Dealership::select(params.rif, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("dealership".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the dealership from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: fetched_dealership,
    }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdateDealershipPartiallyPayload {
    rif: MaybeAbsent<String>,
    name: MaybeAbsent<String>,
    city_number: MaybeAbsent<i32>,
    state_id: MaybeAbsent<i32>,
    manager_national_id: MaybeAbsent<String>,
}

#[patch("/")]
async fn update_dealership_partially(
    Query(params): Query<DealershipManipulationParams>,
    Json(payload): Json<UpdateDealershipPartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let dealership_to_update =
        Dealership::select(params.rif, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("dealership".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err)
                        .context("Failed to fetch the dealership to update from the database"),
                ),
            })?;

    let updated_dealership = UpdateDealership {
        rif: payload.rif.into(),
        name: payload.name.into(),
        city_number: payload.city_number.into(),
        state_id: payload.state_id.into(),
        manager_national_id: payload.manager_national_id.into(),
    }
    .update(dealership_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified rif already exists or the specified managerNationalId is already being used".to_string(), 
                anyhow!(err),
            )
        }
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified cityNumber, stateId or managerNationalId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the dealership from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_dealership,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateDealershipCompletelyPayload {
    rif: String,
    name: String,
    city_number: i32,
    state_id: i32,
    manager_national_id: String,
}

#[put("/")]
async fn update_dealership_completely(
    Query(params): Query<DealershipManipulationParams>,
    Json(payload): Json<UpdateDealershipCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let city_to_update = Dealership::select(params.rif, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("dealership".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the dealership to update from the database"),
            ),
        })?;

    let updated_dealership = UpdateDealership {
        rif: Some(payload.rif),
        name: Some(payload.name),
        city_number: Some(payload.city_number),
        state_id: Some(payload.state_id),
        manager_national_id: Some(payload.manager_national_id),
    }
    .update(city_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified rif already exists or the specified managerNationalId is already being used".to_string(), 
                anyhow!(err),
            )
        }
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified cityNumber, stateId or managerNationalId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the dealership from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_dealership,
    }))
}

#[delete("/")]
async fn delete_dealership(
    Query(params): Query<DealershipManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_dealership = Dealership::delete(params.rif, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("dealership".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the dealership to delete from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: deleted_dealership,
    }))
}
