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
    models::client::{Client, InsertClient, UpdateClient},
    services::pagination_params::PaginationParams,
    services::responses_dto::*,
    services::service_error::ServiceError,
    utils::{deserialization::MaybeAbsent, pagination::Paginable},
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_clients)
        .service(fetch_client)
        .service(create_client)
        .service(update_client_partially)
        .service(update_client_completely)
        .service(delete_client);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreateClientPayload {
    national_id: String,
    full_name: String,
    main_phone_no: String,
    secondary_phone_no: String,
    email: String,
}

#[post("/clients/")]
async fn create_client(
    Json(payload): Json<CreateClientPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_client = InsertClient {
        national_id: payload.national_id,
        full_name: payload.full_name,
        main_phone_no: payload.main_phone_no,
        secondary_phone_no: payload.secondary_phone_no,
        email: payload.email,
    }
    .insert(db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ServiceError::InvalidCreateError(
                "The specified nationalId already exists".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to create the client from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: created_client,
    }))
}

#[get("/clients/")]
async fn fetch_clients(
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

        let fetched_clients = fetch_clients_paginated(per_page, page_no, db.get_ref()).await?;

        let total_clients = Client::count(db.get_ref())
            .await
            .context("Failed to count the clients from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_clients,
                pagination: Pagination::new(total_clients, page_no, per_page),
            });

        return Ok(response);
    }

    let fetched_clients = fetch_all_clients(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_clients,
        });

    Ok(response)
}

async fn fetch_all_clients(db: &Pool<Postgres>) -> Result<Vec<Client>, ServiceError> {
    let fetched_clients = Client::select_all(db)
        .await
        .context("Failed to fetch the clients from the database")?;
    Ok(fetched_clients)
}

async fn fetch_clients_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<Client>, ServiceError> {
    let fetched_clients = Client::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the clients from the database for the provided page")?;

    Ok(fetched_clients.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct ClientManipulationParams {
    national_id: String,
}

#[get("/clients/view/")]
async fn fetch_client(
    Query(params): Query<ClientManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_client = Client::select(params.national_id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("client".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the client from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: fetched_client,
    }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdateClientPartiallyPayload {
    national_id: MaybeAbsent<String>,
    full_name: MaybeAbsent<String>,
    main_phone_no: MaybeAbsent<String>,
    secondary_phone_no: MaybeAbsent<String>,
    email: MaybeAbsent<String>,
}

#[patch("/clients/")]
async fn update_client_partially(
    Query(params): Query<ClientManipulationParams>,
    Json(payload): Json<UpdateClientPartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let city_to_update = Client::select(params.national_id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("client".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the client to update from the database"),
            ),
        })?;

    let updated_client = UpdateClient {
        national_id: payload.national_id.into(),
        full_name: payload.full_name.into(),
        main_phone_no: payload.main_phone_no.into(),
        secondary_phone_no: payload.secondary_phone_no.into(),
        email: payload.email.into(),
    }
    .update(city_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified nationalId already exists".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the client from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_client,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateClientCompletelyPayload {
    national_id: String,
    full_name: String,
    main_phone_no: String,
    secondary_phone_no: String,
    email: String,
}

#[put("/clients/")]
async fn update_client_completely(
    Query(params): Query<ClientManipulationParams>,
    Json(payload): Json<UpdateClientCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let city_to_update = Client::select(params.national_id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("client".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the client to update from the database"),
            ),
        })?;

    let updated_client = UpdateClient {
        national_id: Some(payload.national_id),
        full_name: Some(payload.full_name),
        main_phone_no: Some(payload.main_phone_no),
        secondary_phone_no: Some(payload.secondary_phone_no),
        email: Some(payload.email),
    }
    .update(city_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified nationalId already exists".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the client from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_client,
    }))
}

#[delete("/clients/")]
async fn delete_client(
    Query(params): Query<ClientManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_client = Client::delete(params.national_id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("client".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to get the client to delete from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: deleted_client,
    }))
}
