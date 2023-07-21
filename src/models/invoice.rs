use async_trait::async_trait;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use time::Date;

use crate::utils::pagination::{Page, Pages, Paginable};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
    pub id: i32,
    pub order_id: i32,
    pub amount_due: BigDecimal,
    pub discount: BigDecimal,
    pub issue_date: Date
}

impl Invoice {
    pub async fn select(
        id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Invoice, sqlx::Error> {
        sqlx::query_as!(
            Invoice,
            r#"
            SELECT
                id,
                order_id,
                amount_due,
                discount,
                issue_date
            FROM invoices
            WHERE id = $1
            "#,
            id,
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<Invoice>, sqlx::Error> {
        sqlx::query_as!(
            Invoice,
            r#"
            SELECT
                id,
                order_id,
                amount_due,
                discount,
                issue_date
            FROM invoices
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
            SELECT COUNT(*) AS "total_invoices!"
            FROM invoices
            "#
        )
        .fetch_one(connection)
        .await
    }

    pub async fn delete(
        id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Invoice, sqlx::Error> {
        sqlx::query_as!(
            Invoice,
            r#"
            DELETE FROM invoices
            WHERE id = $1
            RETURNING
                id,
                order_id,
                amount_due,
                discount,
                issue_date
            "#,
            id,
        )
        .fetch_one(connection)
        .await
    }
}

#[async_trait]
impl Paginable<Invoice> for Invoice {
    async fn get_page(
        pages: &Pages<Invoice, Invoice>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<Invoice>, sqlx::Error> {
        let page_items = sqlx::query_as!(
            Invoice,
            r#"
                SELECT
                    id,
                    order_id,
                    amount_due,
                    discount,
                    issue_date
                FROM invoices
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
pub struct InsertInvoice {
    pub order_id: i32,
    pub issue_date: Date
}

impl InsertInvoice {
    pub async fn insert(
        self,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Invoice, sqlx::Error> {
        sqlx::query_as!(
            Invoice,
            r#"
            INSERT INTO invoices (
                order_id,
                issue_date
            )
            VALUES (
                $1,
                $2
            )
            RETURNING
                id,
                order_id,
                amount_due,
                discount,
                issue_date
            "#,
            self.order_id,
            self.issue_date
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateInvoice {
    pub order_id: Option<i32>,
    pub issue_date: Option<Date>
}

impl UpdateInvoice {
    pub async fn update(
        self,
        target: Invoice,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Invoice, sqlx::Error> {
        let new_order_id = self.order_id.unwrap_or(target.order_id);
        let new_issue_date = self.issue_date.unwrap_or(target.issue_date);

        sqlx::query_as!(
            Invoice,
            r#"
            UPDATE invoices
            SET
                order_id = $1,
                issue_date = $2
            WHERE id = $3
            RETURNING
                id,
                order_id,
                amount_due,
                discount,
                issue_date
            "#,
            new_order_id,
            new_issue_date,
            target.id
        )
        .fetch_one(connection)
        .await
    }
}
