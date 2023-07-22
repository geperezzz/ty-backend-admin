use actix_web::{
    get,
    web::{Data, Json, ServiceConfig},
    Responder,
};
use anyhow::Context;
use sqlx::{Pool, Postgres};

use crate::{
    services::responses_dto::*, services::service_error::ServiceError,
    views::no_show_client::NoShowClient,
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration.service(fetch_no_show_clients);
}

#[get("/")]
async fn fetch_no_show_clients(db: Data<Pool<Postgres>>) -> Result<impl Responder, ServiceError> {
    let fetched_clients = NoShowClient::select_all(db.get_ref())
        .await
        .context("Failed to fetch the clients from the database")?;
    Ok(Json(NonPaginatedResponseDto {
        data: fetched_clients,
    }))
}
