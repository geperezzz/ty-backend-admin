use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use bigdecimal::BigDecimal;

use crate::utils::pagination::{Page, Pages, Paginable};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub activity_number: i32,
    pub service_id: i32,
    pub description: String,
    pub price_per_hour: BigDecimal,
}

impl Activity {
    pub async fn select(
        activity_number: i32,
        service_id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Activity, sqlx::Error> {
        sqlx::query_as!(
            Activity,
            r#"
            SELECT
                activity_number,
                service_id,
                description,
                price_per_hour
            FROM activities
            WHERE
                activity_number = $1
                AND service_id = $2
            "#,
            activity_number,
            service_id
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<Activity>, sqlx::Error> {
        sqlx::query_as!(
            Activity,
            r#"
            SELECT
                activity_number,
                service_id,
                description,
                price_per_hour
            FROM activities
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
            SELECT COUNT(*) AS "total_activities!"
            FROM activities
            "#
        )
        .fetch_one(connection)
        .await
    }

    pub async fn delete(
        activity_number: i32,
        service_id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Activity, sqlx::Error> {
        sqlx::query_as!(
            Activity,
            r#"
            DELETE FROM activities
            WHERE
                activity_number = $1
                AND service_id = $2
            RETURNING
                activity_number,
                service_id,
                description,
                price_per_hour
            "#,
            activity_number,
            service_id
        )
        .fetch_one(connection)
        .await
    }
}

#[async_trait]
impl Paginable<Activity> for Activity {
    async fn get_page(
        pages: &Pages<Activity, Activity>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<Activity>, sqlx::Error> {
        let page_items = sqlx::query_as!(
            Activity,
            r#"
                SELECT
                    activity_number,
                    service_id,
                    description,
                    price_per_hour
                FROM activities
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
pub struct InsertActivity {
    pub service_id: i32,
    pub description: String,
    pub price_per_hour: BigDecimal,
}

impl InsertActivity {
    pub async fn insert(
        self,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Activity, sqlx::Error> {
        sqlx::query_as!(
            Activity,
            r#"
            INSERT INTO activities (
                service_id,
                description,
                price_per_hour
            )
            VALUES (
                $1,
                $2,
                $3
            )
            RETURNING
                activity_number,
                service_id,
                description,
                price_per_hour
            "#,
            self.service_id,
            self.description,
            self.price_per_hour
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateActivity {
    pub service_id: Option<i32>,
    pub description: Option<String>,
    pub price_per_hour: Option<BigDecimal>,
}

impl UpdateActivity {
    pub async fn update(
        self,
        target: Activity,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Activity, sqlx::Error> {
        let new_service_id = self.service_id.unwrap_or(target.service_id);
        let new_description = self.description.unwrap_or(target.description);
        let new_price_per_hour = self.price_per_hour.unwrap_or(target.price_per_hour);

        sqlx::query_as!(
            Activity,
            r#"
            UPDATE activities
            SET
                service_id = $1,
                description = $2,
                price_per_hour = $3
            WHERE
                activity_number = $4
                AND service_id = $5
            RETURNING
                activity_number,
                service_id,
                description,
                price_per_hour
            "#,
            new_service_id,
            new_description,
            new_price_per_hour,
            target.activity_number,
            target.service_id
        )
        .fetch_one(connection)
        .await
    }
}
