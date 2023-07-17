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
    models::state::{State, InsertState, UpdateState},
    services::pagination_params::PaginationParams,
    services::responses_dto::*,
    services::service_error::ServiceError,
    utils::{deserialization::MaybeAbsent, pagination::Paginable},
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_states)
        .service(fetch_state)
        .service(create_state)
        .service(update_state_partially)
        .service(update_state_completely)
        .service(delete_state);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreateStatePayload {
    name: String,
}

#[post("/states/")]
async fn create_state(
    Json(payload): Json<CreateStatePayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_state = InsertState {
        name: payload.name,
    }
    .insert(db.get_ref())
    .await
    .map_err(|err| match &err {
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to create the state from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: created_state,
    }))
}

#[get("/states/")]
async fn fetch_states(
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

        let fetched_states = fetch_states_paginated(per_page, page_no, db.get_ref()).await?;

        let total_states = State::count(db.get_ref())
            .await
            .context("Failed to count the states from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_states,
                pagination: Pagination::new(total_states, page_no, per_page),
            });

        return Ok(response);
    }

    let fetched_states = fetch_all_states(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_states,
        });

    Ok(response)
}

async fn fetch_all_states(db: &Pool<Postgres>) -> Result<Vec<State>, ServiceError> {
    let fetched_states = State::select_all(db)
        .await
        .context("Failed to fetch the states from the database")?;
    Ok(fetched_states)
}

async fn fetch_states_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<State>, ServiceError> {
    let fetched_states = State::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the states from the database for the provided page")?;

    Ok(fetched_states.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct StateManipulationParams {
    id: i32,
}

#[get("/states/view/")]
async fn fetch_state(
    Query(params): Query<StateManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_state = State::select(params.id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("state".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the state from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: fetched_state,
    }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdateStatePartiallyPayload {
    name: MaybeAbsent<String>,
}

#[patch("/states/")]
async fn update_state_partially(
    Query(params): Query<StateManipulationParams>,
    Json(payload): Json<UpdateStatePartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let state_to_update = State::select(params.id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("state".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the state to update from the database"),
            ),
        })?;

    let updated_state = UpdateState {
        name: payload.name.into(),
    }
    .update(state_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the state from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_state,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateStateCompletelyPayload {
    name: String
}

#[put("/states/")]
async fn update_state_completely(
    Query(params): Query<StateManipulationParams>,
    Json(payload): Json<UpdateStateCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let state_to_update = State::select(params.id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("state".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the state to update from the database"),
            ),
        })?;

    let updated_state = UpdateState {
        name: Some(payload.name),
    }
    .update(state_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the state from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_state,
    }))
}

#[delete("/states/")]
async fn delete_state(
    Query(params): Query<StateManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_state = State::delete(params.id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("state".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to get the state to delete from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: deleted_state,
    }))
}
