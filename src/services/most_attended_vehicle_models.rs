use actix_web::{
    get,
    web::{Data, Json, Query, ServiceConfig},
    Responder,
};
use anyhow::Context;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use time::Date;

use crate::{
    services::responses_dto::*, services::service_error::ServiceError,
    views::most_attended_vehicle_model::MostAttendedVehicleModel,
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_most_profitable_vehicle_models_in_range)
        .service(fetch_most_profitable_vehicle_models_by_name);
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct FetchMostAttendedVehicleModelsInRangeParams {
    pub from_date: Date,
    pub to_date: Date,
}

#[get("/")]
async fn fetch_most_profitable_vehicle_models_in_range(
    Query(params): Query<FetchMostAttendedVehicleModelsInRangeParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_vehicle_models = MostAttendedVehicleModel::select_all_in_range(
        params.from_date,
        params.to_date,
        db.get_ref(),
    )
    .await
    .context("Failed to fetch the vehicle models from the database")?;
    Ok(Json(NonPaginatedResponseDto {
        data: fetched_vehicle_models,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct FetchMostAttendedVehicleModelsByNameParams {
    pub name: String,
}

#[get("/")]
async fn fetch_most_profitable_vehicle_models_by_name(
    Query(params): Query<FetchMostAttendedVehicleModelsByNameParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_vehicle_models =
        MostAttendedVehicleModel::select_all_by_name(params.name, db.get_ref())
            .await
            .context("Failed to fetch the vehicle models from the database")?;
    Ok(Json(NonPaginatedResponseDto {
        data: fetched_vehicle_models,
    }))
}
