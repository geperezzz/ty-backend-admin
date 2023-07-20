use actix_web::{
    get,
    web::{Data, Json, ServiceConfig},
    Responder,
};
use anyhow::Context;
use sqlx::{Pool, Postgres};

use crate::{
    services::responses_dto::*, services::service_error::ServiceError,
    views::vehicle_applied_service::VehicleAppliedService,
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration.service(fetch_vehicle_applied_services);
}

#[get("/")]
async fn fetch_vehicle_applied_services(
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_vehicles = VehicleAppliedService::select_all(db.get_ref())
        .await
        .context("Failed to fetch the vehicles from the database")?;
    Ok(Json(NonPaginatedResponseDto {
        data: fetched_vehicles,
    }))
}
