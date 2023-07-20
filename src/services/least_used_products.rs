use actix_web::{
    get,
    web::{Data, Json, ServiceConfig},
    Responder,
};
use anyhow::Context;
use sqlx::{Pool, Postgres};

use crate::{
    services::responses_dto::*, services::service_error::ServiceError,
    views::least_used_product::LeastUsedProduct,
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration.service(fetch_least_used_products);
}

#[get("/")]
async fn fetch_least_used_products(
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_products = LeastUsedProduct::select_all(db.get_ref())
        .await
        .context("Failed to fetch the products from the database")?;
    Ok(Json(NonPaginatedResponseDto {
        data: fetched_products,
    }))
}
