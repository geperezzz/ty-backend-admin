use actix_web::{
    get,
    web::{Data, Json, ServiceConfig},
    Responder,
};
use anyhow::Context;
use sqlx::{Pool, Postgres};

use crate::{
    services::responses_dto::*, services::service_error::ServiceError,
    views::most_requested_service::MostRequestedService,
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration.service(fetch_most_requested_services);
}

#[get("/")]
async fn fetch_most_requested_services(
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_services = MostRequestedService::select_all(db.get_ref())
        .await
        .context("Failed to fetch the services from the database")?;
    Ok(Json(NonPaginatedResponseDto {
        data: fetched_services,
    }))
}
