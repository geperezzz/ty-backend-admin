use actix_web::{
    get,
    post,
    put,
    patch,
    delete,
    web::{
        Query,
        Data,
        Json,
        ServiceConfig
    },
    Responder
};
use serde::{Deserialize, Deserializer};
use anyhow::{Context, anyhow};
use sqlx::{
    Pool,
    Postgres
};

use crate::{
    services::service_error::ServiceError,
    models::city::{
        City,
        InsertCity,
        UpdateCity
    }
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_cities)
        .service(fetch_city)
        .service(create_city)
        .service(update_city_partially)
        .service(update_city_completely)
        .service(delete_city);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreateCityPayload {
    name: String,
    state_id: i32
}

#[post("/cities/")]
async fn create_city(
    Json(payload): Json<CreateCityPayload>,
    db: Data<Pool<Postgres>>
) -> Result<impl Responder, ServiceError> {
    let created_city =
        InsertCity {
            name: payload.name,
            state_id: payload.state_id
        }
        .insert(db.get_ref())
        .await
        .map_err(|err| match err {
            sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() =>
                ServiceError::InvalidCreateError("The specified state_id does not exist".to_string()),
            _ =>
                ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to insert the city into the database")
                )
        })?;

    Ok(Json(created_city))
}

#[get("/cities/view/")]
async fn fetch_cities(db: Data<Pool<Postgres>>) -> Result<impl Responder, ServiceError> {
    let fetched_cities = City::select_all(db.get_ref())
        .await
        .context("Failed to fetch the cities from the database")?;
    Ok(Json(fetched_cities))
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct CityManipulationParams {
    city_number: i32,
    state_id: i32
}

#[get("/cities/view/")]
async fn fetch_city(
    Query(params): Query<CityManipulationParams>,
    db: Data<Pool<Postgres>>
) -> Result<impl Responder, ServiceError> {
    let fetched_city = City::select(params.city_number, params.state_id, db.get_ref())
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => ServiceError::ResourceNotFound("city".to_string()),
            _ =>
                ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to get the city from the database")
                )
        })?;

    Ok(Json(fetched_city))
}

fn deserialize_as_inner<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>
{
    Ok(Some(T::deserialize(deserializer)?))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdateCityPartiallyPayload {
    #[serde(deserialize_with = "deserialize_as_inner")]
    name: Option<String>,
    #[serde(deserialize_with = "deserialize_as_inner")]
    state_id: Option<i32>
}

#[patch("/cities/")]
async fn update_city_partially(
    Query(params): Query<CityManipulationParams>,
    Json(payload): Json<UpdateCityPartiallyPayload>,
    db: Data<Pool<Postgres>>
) -> Result<impl Responder, ServiceError> {
    let city_to_update = City::select(params.city_number, params.state_id, db.get_ref())
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => ServiceError::ResourceNotFound("city".to_string()),
            _ =>
                ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to get the city to update from the database")
                )
        })?;
        
    let updated_city = 
        UpdateCity {
            name: payload.name,
            state_id: payload.state_id
        }
        .update(city_to_update, db.get_ref())
        .await
        .map_err(|err| match err {
            sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() =>
                ServiceError::InvalidUpdateError("The specified state_id does not exist".to_string()),
            _ =>
                ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to update the city from the database")
                )
        })?;

    Ok(Json(updated_city))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateCityCompletelyPayload {
    name: String,
    state_id: i32
}

#[put("/cities/")]
async fn update_city_completely(
    Query(params): Query<CityManipulationParams>,
    Json(payload): Json<UpdateCityCompletelyPayload>,
    db: Data<Pool<Postgres>>
) -> Result<impl Responder, ServiceError> {
    let city_to_update = City::select(params.city_number, params.state_id, db.get_ref())
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => ServiceError::ResourceNotFound("city".to_string()),
            _ =>
                ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to get the city to update from the database")
                )
        })?;

    let updated_city = 
        UpdateCity {
            name: Some(payload.name),
            state_id: Some(payload.state_id)
        }
        .update(city_to_update, db.get_ref())
        .await
        .map_err(|err| match err {
            sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() =>
                ServiceError::InvalidUpdateError("The specified state_id does not exist".to_string()),
            _ =>
                ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to update the city from the database")
                )
        })?;

    Ok(Json(updated_city))
}

#[delete("/cities/")]
async fn delete_city(
    Query(params): Query<CityManipulationParams>,
    db: Data<Pool<Postgres>>
) -> Result<impl Responder, ServiceError> {
    let deleted_city = City::delete(params.city_number, params.state_id, db.get_ref())
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => ServiceError::ResourceNotFound("city".to_string()),
            _ =>
                ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to get the city to delete from the database")
                )
        })?;
        
    Ok(Json(deleted_city))
}