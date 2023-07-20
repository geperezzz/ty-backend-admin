use actix_web::{
    delete, get,
    http::{header::ContentType, StatusCode},
    patch, post, put,
    web::{Data, Json, Query, ServiceConfig},
    HttpResponse, Responder,
};
use anyhow::{anyhow, Context};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::{
    models::employee::{Employee, InsertEmployee, UpdateEmployee},
    services::pagination_params::PaginationParams,
    services::responses_dto::*,
    services::service_error::ServiceError,
    utils::{deserialization::{MaybeAbsent, MaybeNull}, pagination::Paginable},
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_staff)
        .service(fetch_employee)
        .service(create_employee)
        .service(update_employee_partially)
        .service(update_employee_completely)
        .service(delete_employee);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreateEmployeePayload {
    national_id: String,
    full_name: String,
    main_phone_no: String,
    secondary_phone_no: String,
    email: String,
    address: String,
    employer_dealership_rif: String,
    helped_dealership_rif: Option<String>,
    role_id: i32,
    salary: BigDecimal,
}

#[post("/")]
async fn create_employee(
    Json(payload): Json<CreateEmployeePayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_employee = InsertEmployee {
        national_id: payload.national_id,
        full_name: payload.full_name,
        main_phone_no: payload.main_phone_no,
        secondary_phone_no: payload.secondary_phone_no,
        email: payload.email,
        address: payload.address,
        employer_dealership_rif: payload.employer_dealership_rif,
        helped_dealership_rif: payload.helped_dealership_rif,
        role_id: payload.role_id,
        salary: payload.salary,
    }
    .insert(db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ServiceError::InvalidCreateError(
                "The specified nationalId already exists".to_string(),
                anyhow!(err),
            )
        }
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidCreateError(
                "The specified roleId, employerDealershipRif or helpedDealershipRif does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to insert the employee into the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: created_employee,
    }))
}

#[get("/")]
async fn fetch_staff(
    Query(pagination_params): Query<PaginationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<HttpResponse, ServiceError> {
    if pagination_params.per_page.is_some() && pagination_params.page_no.is_none() {
        return Err(ServiceError::MissingQueryParamError(
            "Missing query param page-no".to_string(),
        ));
    }

    if pagination_params.per_page.is_none() && pagination_params.page_no.is_some() {
        return Err(ServiceError::MissingQueryParamError(
            "Missing query param per-page".to_string(),
        ));
    }

    if pagination_params.per_page.is_some() && pagination_params.page_no.is_some() {
        let (per_page, page_no) = (
            pagination_params.per_page.unwrap(),
            pagination_params.page_no.unwrap(),
        );

        if page_no <= 0 {
            return Err(ServiceError::InvalidQueryParamValueError(
                "Query param page-no must be greater than 0".to_string(),
            ));
        }

        if per_page <= 0 {
            return Err(ServiceError::InvalidQueryParamValueError(
                "Query param per-page must be greater than 0".to_string(),
            ));
        }

        let fetched_staff = fetch_staff_paginated(per_page, page_no, db.get_ref()).await?;

        let total_staff = Employee::count(db.get_ref())
            .await
            .context("Failed to count the staff from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_staff,
                pagination: Pagination::new(total_staff, page_no, per_page),
            });

        return Ok(response);
    }

    let fetched_staff = fetch_all_staff(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_staff,
        });

    Ok(response)
}

async fn fetch_all_staff(db: &Pool<Postgres>) -> Result<Vec<Employee>, ServiceError> {
    let fetched_staff = Employee::select_all(db)
        .await
        .context("Failed to fetch the staff from the database")?;
    Ok(fetched_staff)
}

async fn fetch_staff_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<Employee>, ServiceError> {
    let fetched_staff = Employee::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the staff from the database for the provided page")?;

    Ok(fetched_staff.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct EmployeeManipulationParams {
    national_id: String,
}

#[get("/view/")]
async fn fetch_employee(
    Query(params): Query<EmployeeManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_employee = Employee::select(params.national_id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("employee".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the employee from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: fetched_employee,
    }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdateEmployeePartiallyPayload {
    national_id: MaybeAbsent<String>,
    full_name: MaybeAbsent<String>,
    main_phone_no: MaybeAbsent<String>,
    secondary_phone_no: MaybeAbsent<String>,
    email: MaybeAbsent<String>,
    address: MaybeAbsent<String>,
    employer_dealership_rif: MaybeAbsent<String>,
    helped_dealership_rif: MaybeAbsent<MaybeNull<String>>,
    role_id: MaybeAbsent<i32>,
    salary: MaybeAbsent<BigDecimal>,
}

#[patch("/")]
async fn update_employee_partially(
    Query(params): Query<EmployeeManipulationParams>,
    Json(payload): Json<UpdateEmployeePartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let employee_to_update = Employee::select(params.national_id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("employee".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the employee to update from the database"),
            ),
        })?;

    let updated_employee = UpdateEmployee {
        national_id: payload.national_id.into(),
        full_name: payload.full_name.into(),
        main_phone_no: payload.main_phone_no.into(),
        secondary_phone_no: payload.secondary_phone_no.into(),
        email: payload.email.into(),
        address: payload.address.into(),
        employer_dealership_rif: payload.employer_dealership_rif.into(),
        helped_dealership_rif: payload.helped_dealership_rif.into(),
        role_id: payload.role_id.into(),
        salary: payload.salary.into(),
    }
    .update(employee_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified nationalId already exists".to_string(),
                anyhow!(err),
            )
        }
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified roleId, employerDealershipRif or helpedDealershipRif does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the employee from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_employee,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateEmployeeCompletelyPayload {
    national_id: String,
    full_name: String,
    main_phone_no: String,
    secondary_phone_no: String,
    email: String,
    address: String,
    employer_dealership_rif: String,
    helped_dealership_rif: MaybeNull<String>,
    role_id: i32,
    salary: BigDecimal,
}

#[put("/")]
async fn update_employee_completely(
    Query(params): Query<EmployeeManipulationParams>,
    Json(payload): Json<UpdateEmployeeCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let employee_to_update = Employee::select(params.national_id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("employee".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the employee to update from the database"),
            ),
        })?;

    let updated_employee = UpdateEmployee {
        national_id: Some(payload.national_id),
        full_name: Some(payload.full_name),
        main_phone_no: Some(payload.main_phone_no),
        secondary_phone_no: Some(payload.secondary_phone_no),
        email: Some(payload.email),
        address: Some(payload.address),
        employer_dealership_rif: Some(payload.employer_dealership_rif),
        helped_dealership_rif: Some(payload.helped_dealership_rif.into()),
        role_id: Some(payload.role_id),
        salary: Some(payload.salary),
    }
    .update(employee_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified nationalId already exists".to_string(),
                anyhow!(err),
            )
        }
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified roleId, employerDealershipRif or helpedDealershipRif does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the employee from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_employee,
    }))
}

#[delete("/")]
async fn delete_employee(
    Query(params): Query<EmployeeManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_employee = Employee::delete(params.national_id, db.get_ref())
        .await
        .map_err(|err| match &err {
            sqlx::Error::RowNotFound => {
                ServiceError::ResourceNotFound("employee".to_string(), anyhow!(err))
            }
            _ => ServiceError::UnexpectedError(
                anyhow!(err).context("Failed to fetch the employee to delete from the database"),
            ),
        })?;

    Ok(Json(NonPaginatedResponseDto {
        data: deleted_employee,
    }))
}
