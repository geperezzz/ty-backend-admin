use actix_web::{
    body::EitherBody,
    delete, get,
    http::{header::ContentType, StatusCode},
    patch, post, put,
    web::{Data, Json, JsonBody, Query, ServiceConfig},
    HttpResponse, Responder,
};
use anyhow::{anyhow, Context};
use serde::{Deserialize, Deserializer};
use sqlx::{Pool, Postgres};

use crate::{
    models::city::{City, InsertCity, UpdateCity},
    services::pagination_params::PaginationParams,
    services::responses_dto::*,
    services::service_error::ServiceError,
    utils::pagination::Paginable,
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
    state_id: i32,
}

#[post("/cities/")]
async fn create_city(
    Json(payload): Json<CreateCityPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_city = InsertCity {
        name: payload.name,
        state_id: payload.state_id,
    }
    .insert(db.get_ref())
    .await
    .map_err(|err| match err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidCreateError("The specified stateId does not exist".to_string())
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to insert the city into the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto { data: created_city }))
}

#[get("/cities/")]
async fn fetch_cities(
    Query(pagination_params): Query<PaginationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<HttpResponse, ServiceError> {
    if pagination_params.per_page.is_some() && pagination_params.page_no.is_none() {
        return Err(ServiceError::MissingQueryParamError(
            "Missing query param page-no".to_string()
        ));
    }

    if pagination_params.per_page.is_none() && pagination_params.page_no.is_some() {
        return Err(ServiceError::MissingQueryParamError(
            "Missing query param per-page".to_string()
        ));
    }

    if pagination_params.per_page.is_some() && pagination_params.page_no.is_some() {

        let (per_page, page_no) = (pagination_params.per_page.unwrap(), pagination_params.page_no.unwrap());

        if page_no <= 0 {
            return Err(ServiceError::InvalidQueryParamValueError(
               "Query param page-no must be greater than 0".to_string() 
            ));
        }

        if per_page <= 0 {
            return Err(ServiceError::InvalidQueryParamValueError(
               "Query param per-page must be greater than 0".to_string() 
            ));
        }

        let fetched_cities = fetch_cities_paginated(per_page, page_no, db.get_ref()).await?;

        let total_cities = City::count(db.get_ref())
            .await
            .context("Failed to fetch total cities number from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_cities,
                pagination: Pagination::new(
                    total_cities,
                    page_no,
                    per_page,
                ),
            });

        return Ok(response);
    }

    let fetched_cities = fetch_all_cities(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_cities,
        });

    Ok(response)
}

async fn fetch_all_cities(db: &Pool<Postgres>) -> Result<Vec<City>, ServiceError> {
    let fetched_cities = City::select_all(db)
        .await
        .context("Failed to fetch the cities from the database")?;
    Ok(fetched_cities)
}

async fn fetch_cities_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<City>, ServiceError> {
    let fetched_cities = City::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the cities from the database for the provided page")?;

    Ok(fetched_cities.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct CityManipulationParams {
    city_number: i32,
    state_id: i32,
}

#[get("/cities/view/")]
async fn fetch_city(
    Query(params): Query<CityManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_city = City::select(params.city_number, params.state_id, db.get_ref())
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => ServiceError::ResourceNotFound("city".to_string()),
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to get the city from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto { data: fetched_city }))
}

fn deserialize_as_inner<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
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
    state_id: Option<i32>,
}

#[patch("/cities/")]
async fn update_city_partially(
    Query(params): Query<CityManipulationParams>,
    Json(payload): Json<UpdateCityPartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let city_to_update = City::select(params.city_number, params.state_id, db.get_ref())
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => ServiceError::ResourceNotFound("city".to_string()),
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to get the city to update from the database"),
            ),
        })?;

    let updated_city = UpdateCity {
        name: payload.name,
        state_id: payload.state_id,
    }
    .update(city_to_update, db.get_ref())
    .await
    .map_err(|err| match err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError("The specified stateId does not exist".to_string())
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the city from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto { data: updated_city }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateCityCompletelyPayload {
    name: String,
    state_id: i32,
}

#[put("/cities/")]
async fn update_city_completely(
    Query(params): Query<CityManipulationParams>,
    Json(payload): Json<UpdateCityCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let city_to_update = City::select(params.city_number, params.state_id, db.get_ref())
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => ServiceError::ResourceNotFound("city".to_string()),
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to get the city to update from the database"),
            ),
        })?;

    let updated_city = UpdateCity {
        name: Some(payload.name),
        state_id: Some(payload.state_id),
    }
    .update(city_to_update, db.get_ref())
    .await
    .map_err(|err| match err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError("The specified stateId does not exist".to_string())
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the city from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto { data: updated_city }))
}

#[delete("/cities/")]
async fn delete_city(
    Query(params): Query<CityManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_city = City::delete(params.city_number, params.state_id, db.get_ref())
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => ServiceError::ResourceNotFound("city".to_string()),
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to get the city to delete from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto { data: deleted_city }))
}
