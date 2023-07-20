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
    views::most_profitable_dealership::MostProfitableDealership,
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration.service(fetch_most_profitable_dealerships);
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct FetchMostProfitableDealershipsParams {
    pub from_date: Date,
    pub to_date: Date,
}

#[get("/")]
async fn fetch_most_profitable_dealerships(
    Query(params): Query<FetchMostProfitableDealershipsParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_dealerships = MostProfitableDealership::select_all_in_range(
        params.from_date,
        params.to_date,
        db.get_ref(),
    )
    .await
    .context("Failed to fetch the dealerships from the database")?;
    Ok(Json(NonPaginatedResponseDto {
        data: fetched_dealerships,
    }))
}
