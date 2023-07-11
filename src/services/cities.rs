use actix_web::{
    get,
    post,
    put,
    delete,
    web::{
        Path,
        Data,
        Json,
        ServiceConfig
    },
    Responder
};
use anyhow::Context;
use sqlx::{
    Pool,
    Postgres
};

use crate::{
    services::service_error::ServiceError,
    models::city::{
        self,
        City
    }
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_cities)
        .service(fetch_city)
        .service(create_city)
        .service(update_city)
        .service(delete_city);
}

#[post("?")]
async fn create_city(state_id: Path<i32>, Json(city): Json<City>, db: Data<Pool<Postgres>>) -> Result<impl Responder, ServiceError> {
    let inserted_city = city::insert(state_id, &city, db.get_ref())
        .await
        .context("Failed to insert the city into the database")?;
    Ok(Json(inserted_city))
}

#[get("/cities/")]
async fn fetch_cities(db: Data<Pool<Postgres>>) -> Result<impl Responder, ServiceError> {
    let fetched_cities = city::select_all(db.get_ref())
        .await
        .context("Failed to fetch the cities from the database")?;
    Ok(Json(fetched_cities))
}

#[get("/cities/{id}/")]
async fn fetch_city(id: Path<i32>, db: Data<Pool<Postgres>>) -> Result<impl Responder, ServiceError> {
    let fetched_city = city::select(*id, db.get_ref())
        .await
        .context("Failed to fetch the city from the database")?;
    Ok(Json(fetched_city))
}

#[put("/cities/{id}/")]
async fn update_city(id: Path<i32>, Json(city): Json<City>, db: Data<Pool<Postgres>>) -> Result<impl Responder, ServiceError> {
    let city = City { id: *id, ..city };
    let updated_city = city::update(&city, db.get_ref())
        .await
        .context("Failed to update the city from the database")?;
    Ok(Json(updated_city))
}

#[delete("/cities/{id}/")]
async fn delete_city(id: Path<i32>, db: Data<Pool<Postgres>>) -> Result<impl Responder, ServiceError> {
    let deleted_city = city::delete(*id, db.get_ref())
        .await
        .context("Failed to delete the city from the database")?;
    Ok(Json(deleted_city))
}