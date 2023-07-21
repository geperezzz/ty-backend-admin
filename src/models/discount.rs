use async_trait::async_trait;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};

use crate::utils::pagination::{Page, Pages, Paginable};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Discount {
    pub discount_number: i32,
    pub dealership_rif: String,
    pub discount_percentage: BigDecimal,
    pub required_annual_service_usage_count: i16,
}

impl Discount {
    pub async fn select(
        discount_number: i32,
        dealership_rif: String,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Discount, sqlx::Error> {
        sqlx::query_as!(
            Discount,
            r#"
            SELECT 
                discount_number,
                dealership_rif,
                discount_percentage,
                required_annual_service_usage_count
            FROM 
                discounts
            WHERE
                discount_number = $1
                AND dealership_rif = $2
            "#,
            discount_number,
            dealership_rif
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<Discount>, sqlx::Error> {
        sqlx::query_as!(
            Discount,
            r#"
            SELECT
                discount_number,
                dealership_rif,
                discount_percentage,
                required_annual_service_usage_count
            FROM 
                discounts
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
            SELECT 
                COUNT(*) AS "total_discounts!"
            FROM 
                discounts
            "#
        )
        .fetch_one(connection)
        .await
    }

    pub async fn delete(
        discount_number: i32,
        dealership_rif: String,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Discount, sqlx::Error> {
        sqlx::query_as!(
            Discount,
            r#"
            DELETE FROM discounts
            WHERE
                discount_number = $1
                AND dealership_rif = $2
            RETURNING 
                discount_number,
                dealership_rif,
                discount_percentage,
                required_annual_service_usage_count
            "#,
            discount_number,
            dealership_rif
        )
        .fetch_one(connection)
        .await
    }
}

#[async_trait]
impl Paginable<Discount> for Discount {
    async fn get_page(
        pages: &Pages<Discount, Discount>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<Discount>, sqlx::Error> {
        let page_items = sqlx::query_as!(
            Discount,
            r#"
                SELECT 
                    discount_number,
                    dealership_rif,
                    discount_percentage,
                    required_annual_service_usage_count
                FROM 
                    discounts
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
pub struct InsertDiscount {
    pub dealership_rif: String,
    pub discount_percentage: BigDecimal,
    pub required_annual_service_usage_count: i16,
}

impl InsertDiscount {
    pub async fn insert(
        self,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Discount, sqlx::Error> {
        sqlx::query_as!(
            Discount,
            r#"
            INSERT INTO discounts (
                dealership_rif,
                discount_percentage,
                required_annual_service_usage_count
            )
            VALUES (
                $1,
                $2,
                $3
            )
            RETURNING 
                discount_number,
                dealership_rif,
                discount_percentage,
                required_annual_service_usage_count
            "#,
            self.dealership_rif as _,
            self.discount_percentage,
            self.required_annual_service_usage_count,
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateDiscount {
    pub dealership_rif: Option<String>,
    pub discount_percentage: Option<BigDecimal>,
    pub required_annual_service_usage_count: Option<i16>,
}

impl UpdateDiscount {
    pub async fn update(
        self,
        target: Discount,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Discount, sqlx::Error> {
        let new_dealership_rif = self
            .dealership_rif
            .as_ref()
            .unwrap_or(&target.dealership_rif);
        let new_discount_percentage = self
            .discount_percentage
            .unwrap_or(target.discount_percentage);
        let new_required_annual_service_usage_count = self
            .required_annual_service_usage_count
            .unwrap_or(target.required_annual_service_usage_count);

        sqlx::query_as!(
            Discount,
            r#"
            UPDATE discounts
            SET 
                dealership_rif = $1,
                discount_percentage = $2,
                required_annual_service_usage_count = $3
            WHERE 
                discount_number = $4
                AND dealership_rif = $5
            RETURNING 
                discount_number,
                dealership_rif,
                discount_percentage,
                required_annual_service_usage_count
            "#,
            new_dealership_rif as _,
            new_discount_percentage,
            new_required_annual_service_usage_count,
            target.discount_number,
            target.dealership_rif as _,
        )
        .fetch_one(connection)
        .await
    }
}
