use actix_web::{
    delete, get,
    http::{header::ContentType, StatusCode},
    patch, post, put,
    web::{Data, Json, Query, ServiceConfig},
    HttpResponse, Responder,
};
use anyhow::{anyhow, Context};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use time::Date;

use crate::{
    models::invoice::{Invoice, InsertInvoice, UpdateInvoice},
    services::pagination_params::PaginationParams,
    services::responses_dto::*,
    services::service_error::ServiceError,
    utils::{deserialization::MaybeAbsent, pagination::Paginable},
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_invoices)
        .service(fetch_invoice)
        .service(create_invoice)
        .service(update_invoice_partially)
        .service(update_invoice_completely)
        .service(delete_invoice);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreateInvoicePayload {
    order_id: i32,
    issue_date: Date
}

#[post("/")]
async fn create_invoice(
    Json(payload): Json<CreateInvoicePayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_invoice = InsertInvoice {
        order_id: payload.order_id,
        issue_date: payload.issue_date,
    }
    .insert(db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidCreateError(
                "The specified orderId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to insert the invoice into the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: created_invoice,
    }))
}

#[get("/")]
async fn fetch_invoices(
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

        let fetched_invoices = fetch_invoices_paginated(per_page, page_no, db.get_ref()).await?;

        let total_invoices = Invoice::count(db.get_ref())
            .await
            .context("Failed to count the invoices from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_invoices,
                pagination: Pagination::new(total_invoices, page_no, per_page),
            });

        return Ok(response);
    }

    let fetched_invoices = fetch_all_invoices(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_invoices,
        });

    Ok(response)
}

async fn fetch_all_invoices(db: &Pool<Postgres>) -> Result<Vec<Invoice>, ServiceError> {
    let fetched_invoices = Invoice::select_all(db)
        .await
        .context("Failed to fetch the invoices from the database")?;
    Ok(fetched_invoices)
}

async fn fetch_invoices_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<Invoice>, ServiceError> {
    let fetched_invoices = Invoice::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the invoices from the database for the provided page")?;

    Ok(fetched_invoices.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct InvoiceManipulationParams {
    id: i32
}

#[get("/view/")]
async fn fetch_invoice(
    Query(params): Query<InvoiceManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_invoice =
        Invoice::select(params.id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("invoice".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the invoice from the database"),
                ),
            })?;

    Ok(Json(NonPaginatedResponseDto {
        data: fetched_invoice,
    }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdateInvoicePartiallyPayload {
    order_id: MaybeAbsent<i32>,
    issue_date: MaybeAbsent<Date>
}

#[patch("/")]
async fn update_invoice_partially(
    Query(params): Query<InvoiceManipulationParams>,
    Json(payload): Json<UpdateInvoicePartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let dealership_to_update =
        Invoice::select(params.id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("invoice".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err)
                        .context("Failed to fetch the invoice to update from the database"),
                ),
            })?;

    let updated_invoice = UpdateInvoice {
        order_id: payload.order_id.into(),
        issue_date: payload.issue_date.into(),
    }
    .update(dealership_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified orderId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the invoice from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_invoice,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdateInvoiceCompletelyPayload {
    order_id: i32,
    issue_date: Date
}

#[put("/")]
async fn update_invoice_completely(
    Query(params): Query<InvoiceManipulationParams>,
    Json(payload): Json<UpdateInvoiceCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let city_to_update =
        Invoice::select(params.id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("invoice".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err)
                        .context("Failed to fetch the invoice to update from the database"),
                ),
            })?;

    let updated_invoice = UpdateInvoice {
        order_id: Some(payload.order_id),
        issue_date: Some(payload.issue_date),
    }
    .update(city_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified orderId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the invoice from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_invoice,
    }))
}

#[delete("/")]
async fn delete_invoice(
    Query(params): Query<InvoiceManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_invoice =
        Invoice::delete(params.id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("invoice".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err)
                        .context("Failed to fetch the invoice to delete from the database"),
                ),
            })?;

    Ok(Json(NonPaginatedResponseDto {
        data: deleted_invoice,
    }))
}
