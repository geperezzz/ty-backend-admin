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
    models::role::{InsertRole, Role, UpdateRole},
    services::pagination_params::PaginationParams,
    services::responses_dto::*,
    services::service_error::ServiceError,
    utils::{deserialization::MaybeAbsent, pagination::Paginable},
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_roles)
        .service(fetch_role)
        .service(create_role)
        .service(update_role_partially)
        .service(update_role_completely)
        .service(delete_role);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreateRolePayload {
    name: String,
    description: String,
}

#[post("/roles/")]
async fn create_role(
    Json(payload): Json<CreateRolePayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_role = InsertRole {
        name: payload.name,
        description: payload.description,
    }
    .insert(db.get_ref())
    .await
    .context("Failed to insert the role into the database")?;

    Ok(Json(NonPaginatedResponseDto { data: created_role }))
}

#[get("/roles/")]
async fn fetch_roles(
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

        let fetched_roles = fetch_roles_paginated(per_page, page_no, db.get_ref()).await?;

        let total_roles = Role::count(db.get_ref())
            .await
            .context("Failed to count the roles from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_roles,
                pagination: Pagination::new(total_roles, page_no, per_page),
            });

        return Ok(response);
    }

    let fetched_roles = fetch_all_roles(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_roles,
        });

    Ok(response)
}

async fn fetch_all_roles(db: &Pool<Postgres>) -> Result<Vec<Role>, ServiceError> {
    let fetched_roles = Role::select_all(db)
        .await
        .context("Failed to fetch the roles from the database")?;
    Ok(fetched_roles)
}

async fn fetch_roles_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<Role>, ServiceError> {
    let fetched_roles = Role::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the roles from the database for the provided page")?;

    Ok(fetched_roles.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct RoleManipulationParams {
    id: i32,
}

#[get("/roles/view/")]
async fn fetch_role(
    Query(params): Query<RoleManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_role = Role::select(params.id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("role".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the role from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto { data: fetched_role }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdateRolePartiallyPayload {
    name: MaybeAbsent<String>,
    description: MaybeAbsent<String>,
}

#[patch("/roles/")]
async fn update_role_partially(
    Query(params): Query<RoleManipulationParams>,
    Json(payload): Json<UpdateRolePartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let role_to_update = Role::select(params.id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("role".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the role to update from the database"),
            ),
        })?;

    let updated_role = UpdateRole {
        name: payload.name.into(),
        description: payload.description.into(),
    }
    .update(role_to_update, db.get_ref())
    .await
    .context("Failed to update the role from the database")?;

    Ok(Json(NonPaginatedResponseDto { data: updated_role }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateRoleCompletelyPayload {
    name: String,
    description: String,
}

#[put("/roles/")]
async fn update_role_completely(
    Query(params): Query<RoleManipulationParams>,
    Json(payload): Json<UpdateRoleCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let role_to_update = Role::select(params.id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("role".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the roles to update from the database"),
            ),
        })?;

    let updated_role = UpdateRole {
        name: Some(payload.name),
        description: Some(payload.description),
    }
    .update(role_to_update, db.get_ref())
    .await
    .context("Failed to update the role from the database")?;

    Ok(Json(NonPaginatedResponseDto { data: updated_role }))
}

#[delete("/roles/")]
async fn delete_role(
    Query(params): Query<RoleManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_role = Role::delete(params.id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("role".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the roles to delete from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto { data: deleted_role }))
}
