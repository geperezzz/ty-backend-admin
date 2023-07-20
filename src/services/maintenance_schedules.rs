use actix_web::{
    get,
    web::{Data, Json, ServiceConfig},
    Responder,
};
use anyhow::Context;
use sqlx::{Pool, Postgres};

use crate::{
    services::responses_dto::*, services::service_error::ServiceError,
    views::maintenance_schedule::MaintenanceSchedule,
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration.service(fetch_maintenance_schedules);
}

#[get("/")]
async fn fetch_maintenance_schedules(
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_maintenance_schedules = MaintenanceSchedule::select_all(db.get_ref())
        .await
        .context("Failed to fetch the maintenance schedules from the database")?;
    Ok(Json(NonPaginatedResponseDto {
        data: fetched_maintenance_schedules,
    }))
}
