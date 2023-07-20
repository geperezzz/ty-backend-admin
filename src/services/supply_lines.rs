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
    models::supply_line::{InsertSupplyLine, SupplyLine, UpdateSupplyLine},
    services::pagination_params::PaginationParams,
    services::responses_dto::*,
    services::service_error::ServiceError,
    utils::{deserialization::MaybeAbsent, pagination::Paginable},
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_supply_lines)
        .service(fetch_supply_line)
        .service(create_supply_line)
        .service(update_supply_line_partially)
        .service(update_supply_line_completely)
        .service(delete_supply_line);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreateSupplyLinePayload {
    name: String,
}

#[post("/supply-lines/")]
async fn create_supply_line(
    Json(payload): Json<CreateSupplyLinePayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_supply_line = InsertSupplyLine { name: payload.name }
        .insert(db.get_ref())
        .await
        .context("Failed to insert the supply line into the database")?;

    Ok(Json(NonPaginatedResponseDto {
        data: created_supply_line,
    }))
}

#[get("/supply-lines/")]
async fn fetch_supply_lines(
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

        let fetched_supply_lines =
            fetch_supply_lines_paginated(per_page, page_no, db.get_ref()).await?;

        let total_supply_lines = SupplyLine::count(db.get_ref())
            .await
            .context("Failed to count the supply lines from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_supply_lines,
                pagination: Pagination::new(total_supply_lines, page_no, per_page),
            });

        return Ok(response);
    }

    let fetched_supply_lines = fetch_all_supply_lines(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_supply_lines,
        });

    Ok(response)
}

async fn fetch_all_supply_lines(db: &Pool<Postgres>) -> Result<Vec<SupplyLine>, ServiceError> {
    let fetched_supply_lines = SupplyLine::select_all(db)
        .await
        .context("Failed to fetch the supply lines from the database")?;
    Ok(fetched_supply_lines)
}

async fn fetch_supply_lines_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<SupplyLine>, ServiceError> {
    let fetched_supply_lines = SupplyLine::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the supply lines from the database for the provided page")?;

    Ok(fetched_supply_lines.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct SupplyLineManipulationParams {
    id: i32,
}

#[get("/supply-lines/view/")]
async fn fetch_supply_line(
    Query(params): Query<SupplyLineManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_supply_line = SupplyLine::select(params.id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("supply line".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the supply line from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: fetched_supply_line,
    }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdateSupplyLinePartiallyPayload {
    name: MaybeAbsent<String>,
}

#[patch("/supply-lines/")]
async fn update_supply_line_partially(
    Query(params): Query<SupplyLineManipulationParams>,
    Json(payload): Json<UpdateSupplyLinePartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let supply_line_to_update =
        SupplyLine::select(params.id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("supply line".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the supply line to update from the database"),
                ),
            })?;

    let updated_supply_line = UpdateSupplyLine {
        name: payload.name.into(),
    }
    .update(supply_line_to_update, db.get_ref())
    .await
    .context("Failed to update the supply line from the database")?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_supply_line,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateSupplyLineCompletelyPayload {
    name: String,
}

#[put("/supply-lines/")]
async fn update_supply_line_completely(
    Query(params): Query<SupplyLineManipulationParams>,
    Json(payload): Json<UpdateSupplyLineCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let supply_line_to_update =
        SupplyLine::select(params.id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("supply line".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the supply line to update from the database"),
                ),
            })?;

    let updated_supply_line = UpdateSupplyLine {
        name: Some(payload.name),
    }
    .update(supply_line_to_update, db.get_ref())
    .await
    .context("Failed to update the supply line from the database")?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_supply_line,
    }))
}

#[delete("/supply-lines/")]
async fn delete_supply_line(
    Query(params): Query<SupplyLineManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_supply_line = SupplyLine::delete(params.id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("supply line".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the supply line to delete from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: deleted_supply_line,
    }))
}
