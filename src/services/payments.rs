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
use time::Date;

use crate::{
    models::payment::{InsertPayment, Payment, UpdatePayment},
    services::pagination_params::PaginationParams,
    services::responses_dto::*,
    services::service_error::ServiceError,
    utils::{deserialization::MaybeAbsent, pagination::Paginable},
};

pub fn configure(configuration: &mut ServiceConfig) {
    configuration
        .service(fetch_payments)
        .service(fetch_payment)
        .service(create_payment)
        .service(update_payment_partially)
        .service(update_payment_completely)
        .service(delete_payment);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CreatePaymentPayload {
    invoice_id: i32,
    amount_paid: BigDecimal,
    payment_date: Date,
    payment_type: String,
    card_number: String,
    card_bank: String
}

#[post("/")]
async fn create_payment(
    Json(payload): Json<CreatePaymentPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let created_payment = InsertPayment {
        invoice_id: payload.invoice_id,
        amount_paid: payload.amount_paid,
        payment_date: payload.payment_date,
        payment_type: payload.payment_type,
        card_number: payload.card_number,
        card_bank: payload.card_bank
    }
    .insert(db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidCreateError(
                "The specified invoiceId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to insert the payment into the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: created_payment,
    }))
}

#[get("/")]
async fn fetch_payments(
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

        let fetched_payments = fetch_payments_paginated(per_page, page_no, db.get_ref()).await?;

        let total_payments = Payment::count(db.get_ref())
            .await
            .context("Failed to count the products from the database")?;

        let response = HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::json())
            .json(PaginatedResponseDto {
                data: fetched_payments,
                pagination: Pagination::new(total_payments, page_no, per_page),
            });

        return Ok(response);
    }

    let fetched_payments = fetch_all_payments(db.get_ref()).await?;

    let response = HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .json(NonPaginatedResponseDto {
            data: fetched_payments,
        });

    Ok(response)
}

async fn fetch_all_payments(db: &Pool<Postgres>) -> Result<Vec<Payment>, ServiceError> {
    let fetched_payments = Payment::select_all(db)
        .await
        .context("Failed to fetch the payments from the database")?;
    Ok(fetched_payments)
}

async fn fetch_payments_paginated(
    per_page: i64,
    page_no: i64,
    db: &Pool<Postgres>,
) -> Result<Vec<Payment>, ServiceError> {
    let fetched_payments = Payment::paginate(per_page)
        .get_page(page_no, db)
        .await
        .context("Failed to fetch the payments from the database for the provided page")?;

    Ok(fetched_payments.items)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct PaymentManipulationParams {
    payment_number: i32,
    invoice_id: i32
}

#[get("/view/")]
async fn fetch_payment(
    Query(params): Query<PaymentManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let fetched_payment =
        Payment::select(params.payment_number, params.invoice_id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("payment".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the payment from the database"),
                ),
            })?;

    Ok(Json(NonPaginatedResponseDto {
        data: fetched_payment,
    }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct UpdatePaymentPartiallyPayload {
    invoice_id: MaybeAbsent<i32>,
    amount_paid: MaybeAbsent<BigDecimal>,
    payment_date: MaybeAbsent<Date>,
    payment_type: MaybeAbsent<String>,
    card_number: MaybeAbsent<String>,
    card_bank: MaybeAbsent<String>
}

#[patch("/")]
async fn update_payment_partially(
    Query(params): Query<PaymentManipulationParams>,
    Json(payload): Json<UpdatePaymentPartiallyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let city_to_update =
        Payment::select(params.payment_number, params.invoice_id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("payment".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the payment to update from the database"),
                ),
            })?;

    let updated_payment = UpdatePayment {
        invoice_id: payload.invoice_id.into(),
        amount_paid: payload.amount_paid.into(),
        payment_date: payload.payment_date.into(),
        payment_type: payload.payment_type.into(),
        card_number: payload.card_number.into(),
        card_bank: payload.card_bank.into(),
    }
    .update(city_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified invoiceId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the payment from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_payment,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct UpdatePaymentCompletelyPayload {
    invoice_id: i32,
    amount_paid: BigDecimal,
    payment_date: Date,
    payment_type: String,
    card_number: String,
    card_bank: String
}

#[put("/")]
async fn update_payment_completely(
    Query(params): Query<PaymentManipulationParams>,
    Json(payload): Json<UpdatePaymentCompletelyPayload>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let city_to_update =
        Payment::select(params.payment_number, params.invoice_id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("payment".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the payment to update from the database"),
                ),
            })?;

    let updated_payment = UpdatePayment {
        invoice_id: Some(payload.invoice_id),
        amount_paid: Some(payload.amount_paid),
        payment_date: Some(payload.payment_date),
        payment_type: Some(payload.payment_type),
        card_number: Some(payload.card_number),
        card_bank: Some(payload.card_bank),
    }
    .update(city_to_update, db.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            ServiceError::InvalidUpdateError(
                "The specified invoiceId does not exist".to_string(),
                anyhow!(err),
            )
        }
        _ => ServiceError::UnexpectedError(
            anyhow!(err).context("Failed to update the payment from the database"),
        ),
    })?;

    Ok(Json(NonPaginatedResponseDto {
        data: updated_payment,
    }))
}

#[delete("/")]
async fn delete_payment(
    Query(params): Query<PaymentManipulationParams>,
    db: Data<Pool<Postgres>>,
) -> Result<impl Responder, ServiceError> {
    let deleted_payment =
        Payment::delete(params.payment_number, params.invoice_id, db.get_ref())
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => {
                    ServiceError::ResourceNotFound("payment".to_string(), anyhow!(err))
                }
                _ => ServiceError::UnexpectedError(
                    anyhow!(err).context("Failed to fetch the payment to delete from the database"),
                ),
            })?;

    Ok(Json(NonPaginatedResponseDto {
        data: deleted_payment,
    }))
}
