use actix_web::{
    get,
    web::{Data, Json, ServiceConfig},
    Responder,
};
use anyhow::Context;
use sqlx::{Pool, Postgres};

use crate::{
    services::responses_dto::*, services::service_error::ServiceError,
    views::least_employed_employee::LeastEmployedEmployee,
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration.service(fetch_least_employed_employees);
}

#[get("/")]
async fn fetch_least_employed_employees(
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_employees = LeastEmployedEmployee::select_all(db.get_ref())
        .await
        .context("Failed to fetch the employees from the database")?;
    Ok(Json(NonPaginatedResponseDto {
        data: fetched_employees,
    }))
}
