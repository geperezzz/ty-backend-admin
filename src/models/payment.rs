use async_trait::async_trait;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use time::Date;

use crate::utils::pagination::{Page, Pages, Paginable};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payment {
    pub payment_number: i32,
    pub invoice_id: i32,
    pub amount_paid: BigDecimal,
    pub payment_date: Date,
    pub payment_type: String,
    pub card_number: String,
    pub card_bank: String
}

impl Payment {
    pub async fn select(
        payment_number: i32,
        invoice_id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Payment, sqlx::Error> {
        sqlx::query_as!(
            Payment,
            r#"
            SELECT
                payment_number,
                invoice_id,
                amount_paid,
                payment_date,
                payment_type,
                card_number,
                card_bank
            FROM payments
            WHERE
                payment_number = $1
                AND invoice_id = $2
            "#,
            payment_number,
            invoice_id,
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<Payment>, sqlx::Error> {
        sqlx::query_as!(
            Payment,
            r#"
            SELECT
                payment_number,
                invoice_id,
                amount_paid,
                payment_date,
                payment_type,
                card_number,
                card_bank
            FROM payments
            "#
        )
        .fetch_all(connection)
        .await
    }

    pub async fn count(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "total_payments!"
            FROM payments
            "#
        )
        .fetch_one(connection)
        .await
    }

    pub async fn delete(
        payment_number: i32,
        invoice_id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Payment, sqlx::Error> {
        sqlx::query_as!(
            Payment,
            r#"
            DELETE FROM payments
            WHERE
                payment_number = $1
                AND invoice_id = $2
            RETURNING
                payment_number,
                invoice_id,
                amount_paid,
                payment_date,
                payment_type,
                card_number,
                card_bank
            "#,
            payment_number,
            invoice_id
        )
        .fetch_one(connection)
        .await
    }
}

#[async_trait]
impl Paginable<Payment> for Payment {
    async fn get_page(
        pages: &Pages<Payment, Payment>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<Payment>, sqlx::Error> {
        let page_items = sqlx::query_as!(
            Payment,
            r#"
            SELECT
                payment_number,
                invoice_id,
                amount_paid,
                payment_date,
                payment_type,
                card_number,
                card_bank
            FROM payments
            LIMIT $1
            OFFSET $2
            "#,
            pages.per_page,
            (page_no - 1) * pages.per_page
        )
        .fetch_all(connection)
        .await?;

        Ok(Page {
            per_page: pages.per_page,
            page_no,
            items: page_items,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct InsertPayment {
    pub invoice_id: i32,
    pub amount_paid: BigDecimal,
    pub payment_date: Date,
    pub payment_type: String,
    pub card_number: String,
    pub card_bank: String
}

impl InsertPayment {
    pub async fn insert(
        self,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Payment, sqlx::Error> {
        sqlx::query_as!(
            Payment,
            r#"
            INSERT INTO payments (
                invoice_id,
                amount_paid,
                payment_date,
                payment_type,
                card_number,
                card_bank
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6
            )
            RETURNING
                payment_number,
                invoice_id,
                amount_paid,
                payment_date,
                payment_type,
                card_number,
                card_bank
            "#,
            self.invoice_id,
            self.amount_paid,
            self.payment_date,
            self.payment_type,
            self.card_number,
            self.card_bank
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdatePayment {
    pub invoice_id: Option<i32>,
    pub amount_paid: Option<BigDecimal>,
    pub payment_date: Option<Date>,
    pub payment_type: Option<String>,
    pub card_number: Option<String>,
    pub card_bank: Option<String>
}

impl UpdatePayment {
    pub async fn update(
        self,
        target: Payment,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Payment, sqlx::Error> {
        let new_invoice_id = self.invoice_id.unwrap_or(target.invoice_id);
        let new_amount_paid = self.amount_paid.unwrap_or(target.amount_paid);
        let new_payment_date = self.payment_date.unwrap_or(target.payment_date);
        let new_payment_type = self.payment_type.unwrap_or(target.payment_type);
        let new_card_number = self.card_number.unwrap_or(target.card_number);
        let new_card_bank = self.card_bank.unwrap_or(target.card_bank);

        sqlx::query_as!(
            Payment,
            r#"
            UPDATE payments
            SET
                invoice_id = $1,
                amount_paid = $2,
                payment_date = $3,
                payment_type = $4,
                card_number = $5,
                card_bank = $6
            WHERE
                payment_number = $7
                AND invoice_id = $8
            RETURNING
                payment_number,
                invoice_id,
                amount_paid,
                payment_date,
                payment_type,
                card_number,
                card_bank
            "#,
            new_invoice_id,
            new_amount_paid,
            new_payment_date,
            new_payment_type,
            new_card_number,
            new_card_bank,
            target.payment_number,
            target.invoice_id
        )
        .fetch_one(connection)
        .await
    }
}
