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
    models::service::{InsertService, Service, UpdateService},
    services::pagination_params::PaginationParams,
    services::responses_dto::*,
    services::service_error::ServiceError,
    utils::{deserialization::MaybeAbsent, pagination::Paginable},
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_services)
        .service(fetch_service)
        .service(create_service)
        .service(update_service_partially)
        .service(update_service_completely)
        .service(delete_service);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreateServicePayload {
    name: String,
    description: String,
    coordinator_national_id: String,
}

#[post("/")]
async fn create_service(
    Json(payload): Json<CreateServicePayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_service = InsertService { 
        name: payload.name,
        description: payload.description,
        coordinator_national_id: payload.coordinator_national_id 
    }
    .insert(db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidCreateError(
                "The specified coordinatorNationalId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to insert the service into the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: created_service,
    }))
}

#[get("/")]
async fn fetch_services(
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

        let fetched_services =
            fetch_services_paginated(per_page, page_no, db.get_ref()).await?;

        let total_services = Service::count(db.get_ref())
            .await
            .context("Failed to count the services from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_services,
                pagination: Pagination::new(total_services, page_no, per_page),
            });

        return Ok(response);
    }

    let fetched_services = fetch_all_services(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_services,
        });

    Ok(response)
}

async fn fetch_all_services(db: &Pool<Postgres>) -> Result<Vec<Service>, ServiceError> {
    let fetched_services = Service::select_all(db)
        .await
        .context("Failed to fetch the services from the database")?;
    Ok(fetched_services)
}

async fn fetch_services_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<Service>, ServiceError> {
    let fetched_services = Service::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the services from the database for the provided page")?;

    Ok(fetched_services.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct ServiceManipulationParams {
    id: i32,
}

#[get("/view/")]
async fn fetch_service(
    Query(params): Query<ServiceManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_service = Service::select(params.id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("service".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the service from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: fetched_service,
    }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdateServicePartiallyPayload {
    name: MaybeAbsent<String>,
    description: MaybeAbsent<String>,
    coordinator_national_id: MaybeAbsent<String>,
}

#[patch("/")]
async fn update_service_partially(
    Query(params): Query<ServiceManipulationParams>,
    Json(payload): Json<UpdateServicePartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let service_to_update =
        Service::select(params.id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("service".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the service to update from the database"),
                ),
            })?;

    let updated_service = UpdateService {
        name: payload.name.into(),
        description: payload.description.into(),
        coordinator_national_id: payload.coordinator_national_id.into(),
    }
    .update(service_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidCreateError(
                "The specified coordinatorNationalId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to insert the service into the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_service,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateServiceCompletelyPayload {
    name: String,
    description: String,
    coordinator_national_id: String,
}

#[put("/")]
async fn update_service_completely(
    Query(params): Query<ServiceManipulationParams>,
    Json(payload): Json<UpdateServiceCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let service_to_update =
        Service::select(params.id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("service".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the service to update from the database"),
                ),
            })?;

    let updated_service = UpdateService {
        name: Some(payload.name),
        description: Some(payload.description),
        coordinator_national_id: Some(payload.coordinator_national_id),
    }
    .update(service_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidCreateError(
                "The specified coordinatorNationalId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to insert the service into the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_service,
    }))
}

#[delete("/")]
async fn delete_service(
    Query(params): Query<ServiceManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_service = Service::delete(params.id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("service".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the service to delete from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: deleted_service,
    }))
}
