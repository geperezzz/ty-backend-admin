use async_trait::async_trait;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};

use crate::utils::pagination::{Page, Pages, Paginable};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityPrice {
    pub activity_number: i32,
    pub service_id: i32,
    pub dealership_rif: String,
    pub price_per_hour: BigDecimal,
}

impl ActivityPrice {
    pub async fn select(
        activity_number: i32,
        service_id: i32,
        dealership_rif: String,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<ActivityPrice, sqlx::Error> {
        sqlx::query_as!(
            ActivityPrice,
            r#"
            SELECT
                activity_number,
                service_id,
                dealership_rif,
                price_per_hour
            FROM activities_prices
            WHERE
                activity_number = $1
                AND service_id = $2
                AND dealership_rif = $3
            "#,
            activity_number,
            service_id,
            dealership_rif
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<ActivityPrice>, sqlx::Error> {
        sqlx::query_as!(
            ActivityPrice,
            r#"
            SELECT
                activity_number,
                service_id,
                dealership_rif,
                price_per_hour
            FROM activities_prices
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
            SELECT COUNT(*) AS "total_activities_prices!"
            FROM activities_prices
            "#
        )
        .fetch_one(connection)
        .await
    }

    pub async fn delete(
        activity_number: i32,
        service_id: i32,
        dealership_rif: String,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<ActivityPrice, sqlx::Error> {
        sqlx::query_as!(
            ActivityPrice,
            r#"
            DELETE FROM activities_prices
            WHERE
                activity_number = $1
                AND service_id = $2
                AND dealership_rif = $3
            RETURNING
                activity_number,
                service_id,
                dealership_rif,
                price_per_hour
            "#,
            activity_number,
            service_id,
            dealership_rif
        )
        .fetch_one(connection)
        .await
    }
}

#[async_trait]
impl Paginable<ActivityPrice> for ActivityPrice {
    async fn get_page(
        pages: &Pages<ActivityPrice, ActivityPrice>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<ActivityPrice>, sqlx::Error> {
        let page_items = sqlx::query_as!(
            ActivityPrice,
            r#"
                SELECT
                    activity_number,
                    service_id,
                    dealership_rif,
                    price_per_hour
                FROM activities_prices
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
pub struct InsertActivityPrice {
    pub activity_number: i32,
    pub service_id: i32,
    pub dealership_rif: String,
    pub price_per_hour: BigDecimal,
}

impl InsertActivityPrice {
    pub async fn insert(
        self,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<ActivityPrice, sqlx::Error> {
        sqlx::query_as!(
            ActivityPrice,
            r#"
            INSERT INTO activities_prices (
                activity_number,
                service_id,
                dealership_rif,
                price_per_hour
            )
            VALUES (
                $1,
                $2,
                $3,
                $4
            )
            RETURNING
                activity_number,
                service_id,
                dealership_rif,
                price_per_hour
            "#,
            self.activity_number,
            self.service_id,
            self.dealership_rif as _,
            self.price_per_hour
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateActivityPrice {
    pub activity_number: Option<i32>,
    pub service_id: Option<i32>,
    pub dealership_rif: Option<String>,
    pub price_per_hour: Option<BigDecimal>,
}

impl UpdateActivityPrice {
    pub async fn update(
        self,
        target: ActivityPrice,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<ActivityPrice, sqlx::Error> {
        let new_activity_number = self.activity_number.unwrap_or(target.activity_number);
        let new_service_id = self.service_id.unwrap_or(target.service_id);
        let new_dealership_rif = self
            .dealership_rif
            .as_ref()
            .unwrap_or(&target.dealership_rif);
        let new_price_per_hour = self.price_per_hour.unwrap_or(target.price_per_hour);

        sqlx::query_as!(
            ActivityPrice,
            r#"
            UPDATE activities_prices
            SET
                activity_number = $1,
                service_id = $2,
                dealership_rif = $3,
                price_per_hour = $4
            WHERE
                activity_number = $5
                AND service_id = $6
                AND dealership_rif = $7
            RETURNING
                activity_number,
                service_id,
                dealership_rif,
                price_per_hour
            "#,
            new_activity_number,
            new_service_id,
            new_dealership_rif as _,
            new_price_per_hour,
            target.activity_number,
            target.service_id,
            target.dealership_rif as _
        )
        .fetch_one(connection)
        .await
    }
}
