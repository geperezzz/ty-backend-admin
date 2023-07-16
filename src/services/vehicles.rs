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
use time::Date;

use crate::{
    models::vehicle::{InsertVehicle, UpdateVehicle, Vehicle},
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
        .service(fetch_vehicles)
        .service(fetch_vehicle)
        .service(create_vehicle)
        .service(update_vehicle_partially)
        .service(update_vehicle_completely)
        .service(delete_vehicle);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreateVehiclePayload {
    plate: String,
    brand: String,
    model_id: i32,
    serial_no: String,
    engine_serial_no: String,
    color: String,
    purchase_date: Date,
    additional_info: MaybeNull<String>,
    maintenance_summary: MaybeNull<String>,
    owner_national_id: String,
}

#[post("/vehicles/")]
async fn create_vehicle(
    Json(payload): Json<CreateVehiclePayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_vehicle = InsertVehicle {
        plate: payload.plate,
        brand: payload.brand,
        model_id: payload.model_id,
        serial_no: payload.serial_no,
        engine_serial_no: payload.engine_serial_no,
        color: payload.color,
        purchase_date: payload.purchase_date,
        additional_info: payload.additional_info.into(),
        maintenance_summary: payload.maintenance_summary.into(),
        owner_national_id: payload.owner_national_id,
    }
    .insert(db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ServiceError::InvalidCreateError(
                "The specified plate already exists".to_string(),
                anyhow!(err),
            )
        }
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidCreateError(
                "The specified modelId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to create the client from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: created_vehicle,
    }))
}

#[get("/vehicles/")]
async fn fetch_vehicles(
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

        let fetched_vehicles = fetch_vehicles_paginated(per_page, page_no, db.get_ref()).await?;

        let total_vehicles = Vehicle::count(db.get_ref())
            .await
            .context("Failed to count the vehicles from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_vehicles,
                pagination: Pagination::new(total_vehicles, page_no, per_page),
            });

        return Ok(response);
    }

    let fetched_vehicles = fetch_all_vehicles(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_vehicles,
        });

    Ok(response)
}

async fn fetch_all_vehicles(db: &Pool<Postgres>) -> Result<Vec<Vehicle>, ServiceError> {
    let fetched_vehicles = Vehicle::select_all(db)
        .await
        .context("Failed to fetch the vehicles from the database")?;
    Ok(fetched_vehicles)
}

async fn fetch_vehicles_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<Vehicle>, ServiceError> {
    let fetched_vehicles = Vehicle::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the vehicles from the database for the provided page")?;

    Ok(fetched_vehicles.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct VehicleManipulationParams {
    plate: String,
}

#[get("/vehicles/view/")]
async fn fetch_vehicle(
    Query(params): Query<VehicleManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_vehicle = Vehicle::select(params.plate, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("vehicle".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the vehicle from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: fetched_vehicle,
    }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdateVehiclePartiallyPayload {
    plate: MaybeAbsent<String>,
    brand: MaybeAbsent<String>,
    model_id: MaybeAbsent<i32>,
    serial_no: MaybeAbsent<String>,
    engine_serial_no: MaybeAbsent<String>,
    color: MaybeAbsent<String>,
    purchase_date: MaybeAbsent<Date>,
    additional_info: MaybeAbsent<MaybeNull<String>>,
    maintenance_summary: MaybeAbsent<MaybeNull<String>>,
    owner_national_id: MaybeAbsent<String>,
}

#[patch("/vehicles/")]
async fn update_vehicle_partially(
    Query(params): Query<VehicleManipulationParams>,
    Json(payload): Json<UpdateVehiclePartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let city_to_update =
        Vehicle::select(params.plate, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("vehicle".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the vehicle to update from the database"),
                ),
            })?;

    let updated_vehicle = UpdateVehicle {
        plate: payload.plate.into(),
        brand: payload.brand.into(),
        model_id: payload.model_id.into(),
        serial_no: payload.serial_no.into(),
        engine_serial_no: payload.engine_serial_no.into(),
        color: payload.color.into(),
        purchase_date: payload.purchase_date.into(),
        additional_info: payload.additional_info.into(),
        maintenance_summary: payload.maintenance_summary.into(),
        owner_national_id: payload.owner_national_id.into(),
    }
    .update(city_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified plate already exists".to_string(),
                anyhow!(err),
            )
        }
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified modelId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the vehicle from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_vehicle,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateVehicleCompletelyPayload {
    plate: String,
    brand: String,
    model_id: i32,
    serial_no: String,
    engine_serial_no: String,
    color: String,
    purchase_date: Date,
    additional_info: MaybeNull<String>,
    maintenance_summary: MaybeNull<String>,
    owner_national_id: String,
}

#[put("/vehicles/")]
async fn update_vehicle_completely(
    Query(params): Query<VehicleManipulationParams>,
    Json(payload): Json<UpdateVehicleCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let city_to_update =
        Vehicle::select(params.plate, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("vehicle".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the vehicle to update from the database"),
                ),
            })?;

    let updated_vehicle = UpdateVehicle {
        plate: Some(payload.plate),
        brand: Some(payload.brand),
        model_id: Some(payload.model_id),
        serial_no: Some(payload.serial_no),
        engine_serial_no: Some(payload.engine_serial_no),
        color: Some(payload.color),
        purchase_date: Some(payload.purchase_date),
        additional_info: Some(payload.additional_info.into()),
        maintenance_summary: Some(payload.maintenance_summary.into()),
        owner_national_id: Some(payload.owner_national_id),
    }
    .update(city_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified plate already exists".to_string(),
                anyhow!(err),
            )
        }
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified modelId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the vehicle from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_vehicle,
    }))
}

#[delete("/vehicles/")]
async fn delete_vehicle(
    Query(params): Query<VehicleManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_vehicle = Vehicle::delete(params.plate, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("vehicle".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to get the vehicle to delete from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: deleted_vehicle,
    }))
}
