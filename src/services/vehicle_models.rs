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
use bigdecimal::BigDecimal;

use crate::{
    models::vehicle_model::{VehicleModel, InsertVehicleModel, UpdateVehicleModel},
    services::pagination_params::PaginationParams,
    services::responses_dto::*,
    services::service_error::ServiceError,
    utils::{deserialization::MaybeAbsent, pagination::Paginable},
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_vehicle_models)
        .service(fetch_vehicle_model)
        .service(create_vehicle_model)
        .service(update_vehicle_model_partially)
        .service(update_vehicle_model_completely)
        .service(delete_vehicle_model);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreateVehicleModelPayload {
    pub name: String,
    pub seat_count: i32,
    pub weight_in_kg: BigDecimal,
    pub octane_rating: i16,
    pub gearbox_oil_type: String,
    pub engine_oil_type: String,
    pub engine_coolant_type: String,
}

#[post("/vehicle-models/")]
async fn create_vehicle_model(
    Json(payload): Json<CreateVehicleModelPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_vehicle_model = InsertVehicleModel {
        name: payload.name,
        seat_count: payload.seat_count,
        weight_in_kg: payload.weight_in_kg,
        octane_rating: payload.octane_rating,
        gearbox_oil_type: payload.gearbox_oil_type,
        engine_oil_type: payload.engine_oil_type,
        engine_coolant_type: payload.engine_coolant_type,
    }
    .insert(db.get_ref())
    .await
    .map_err(|err| match &err {
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to create the vehicle model from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: created_vehicle_model,
    }))
}

#[get("/vehicle-models/")]
async fn fetch_vehicle_models(
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

        let fetched_vehicle_models = fetch_vehicle_models_paginated(per_page, page_no, db.get_ref()).await?;

        let total_vehicle_models = VehicleModel::count(db.get_ref())
            .await
            .context("Failed to count the vehicle models from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_vehicle_models,
                pagination: Pagination::new(total_vehicle_models, page_no, per_page),
            });

        return Ok(response);
    }

    let fetched_vehicle_models = fetch_all_vehicle_models(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_vehicle_models,
        });

    Ok(response)
}

async fn fetch_all_vehicle_models(db: &Pool<Postgres>) -> Result<Vec<VehicleModel>, ServiceError> {
    let fetched_vehicle_models = VehicleModel::select_all(db)
        .await
        .context("Failed to fetch the vehicle models from the database")?;
    Ok(fetched_vehicle_models)
}

async fn fetch_vehicle_models_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<VehicleModel>, ServiceError> {
    let fetched_vehicle_models = VehicleModel::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the vehicle models from the database for the provided page")?;

    Ok(fetched_vehicle_models.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct VehicleModelManipulationParams {
    id: i32,
}

#[get("/vehicle-models/view/")]
async fn fetch_vehicle_model(
    Query(params): Query<VehicleModelManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_vehicle_model = VehicleModel::select(params.id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("vehicle model".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the vehicle model from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: fetched_vehicle_model,
    }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdateVehicleModelPartiallyPayload {
    name: MaybeAbsent<String>,
    seat_count: MaybeAbsent<i32>,
    weight_in_kg: MaybeAbsent<BigDecimal>,
    octane_rating: MaybeAbsent<i16>,
    gearbox_oil_type: MaybeAbsent<String>,
    engine_oil_type: MaybeAbsent<String>,
    engine_coolant_type: MaybeAbsent<String>,
}

#[patch("/vehicle-models/")]
async fn update_vehicle_model_partially(
    Query(params): Query<VehicleModelManipulationParams>,
    Json(payload): Json<UpdateVehicleModelPartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let vehicle_model_to_update = VehicleModel::select(params.id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("vehicle model".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the vehicle model to update from the database"),
            ),
        })?;

    let updated_vehicle_model = UpdateVehicleModel {
        name: payload.name.into(),
        seat_count: payload.seat_count.into(),
        weight_in_kg: payload.weight_in_kg.into(),
        octane_rating: payload.octane_rating.into(),
        gearbox_oil_type: payload.gearbox_oil_type.into(),
        engine_oil_type: payload.engine_oil_type.into(),
        engine_coolant_type: payload.engine_coolant_type.into(),
    }
    .update(vehicle_model_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the vehicle model from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_vehicle_model,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateVehicleModelCompletelyPayload {
    name: String,
    seat_count: i32,
    weight_in_kg: BigDecimal,
    octane_rating: i16,
    gearbox_oil_type: String,
    engine_oil_type: String,
    engine_coolant_type: String,
}

#[put("/vehicle-models/")]
async fn update_vehicle_model_completely(
    Query(params): Query<VehicleModelManipulationParams>,
    Json(payload): Json<UpdateVehicleModelCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let vehicle_model_to_update = VehicleModel::select(params.id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("vehicle model".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the vehicle model to update from the database"),
            ),
        })?;

    let updated_vehicle_model = UpdateVehicleModel {
        name: Some(payload.name),
        seat_count: Some(payload.seat_count),
        weight_in_kg: Some(payload.weight_in_kg),
        octane_rating: Some(payload.octane_rating),
        gearbox_oil_type: Some(payload.gearbox_oil_type),
        engine_oil_type: Some(payload.engine_oil_type),
        engine_coolant_type: Some(payload.engine_coolant_type),
    }
    .update(vehicle_model_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the vehicle model from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_vehicle_model,
    }))
}

#[delete("/vehicle-models/")]
async fn delete_vehicle_model(
    Query(params): Query<VehicleModelManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_vehicle_model = VehicleModel::delete(params.id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("vehicle model".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to get the vehicle model to delete from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: deleted_vehicle_model,
    }))
}
