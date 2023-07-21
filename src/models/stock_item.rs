use async_trait::async_trait;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};

use crate::utils::pagination::{Page, Pages, Paginable};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockItem {
    pub product_id: i32,
    pub dealership_rif: String,
    pub product_cost: BigDecimal,
    pub product_count: i32,
    pub vendor_name: String,
    pub max_capacity: i32,
    pub min_capacity: i32,
}

impl StockItem {
    pub async fn select(
        product_id: i32,
        dealership_rif: String,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<StockItem, sqlx::Error> {
        sqlx::query_as!(
            StockItem,
            r#"
            SELECT
                product_id,
                dealership_rif,
                product_cost,
                product_count,
                vendor_name,
                max_capacity,
                min_capacity
            FROM stock
            WHERE
                product_id = $1
                AND dealership_rif = $2
            "#,
            product_id,
            dealership_rif
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<StockItem>, sqlx::Error> {
        sqlx::query_as!(
            StockItem,
            r#"
            SELECT
                product_id,
                dealership_rif,
                product_cost,
                product_count,
                vendor_name,
                max_capacity,
                min_capacity
            FROM stock
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
            SELECT COUNT(*) AS "total_stock!"
            FROM stock
            "#
        )
        .fetch_one(connection)
        .await
    }

    pub async fn delete(
        product_id: i32,
        dealership_rif: String,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<StockItem, sqlx::Error> {
        sqlx::query_as!(
            StockItem,
            r#"
            DELETE FROM stock
            WHERE
                product_id = $1
                AND dealership_rif = $2
            RETURNING
                product_id,
                dealership_rif,
                product_cost,
                product_count,
                vendor_name,
                max_capacity,
                min_capacity
            "#,
            product_id,
            dealership_rif
        )
        .fetch_one(connection)
        .await
    }
}

#[async_trait]
impl Paginable<StockItem> for StockItem {
    async fn get_page(
        pages: &Pages<StockItem, StockItem>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<StockItem>, sqlx::Error> {
        let page_items = sqlx::query_as!(
            StockItem,
            r#"
                SELECT
                    product_id,
                    dealership_rif,
                    product_cost,
                    product_count,
                    vendor_name,
                    max_capacity,
                    min_capacity
                FROM stock
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
pub struct InsertStockItem {
    pub product_id: i32,
    pub dealership_rif: String,
    pub product_cost: BigDecimal,
    pub product_count: i32,
    pub vendor_name: String,
    pub max_capacity: i32,
    pub min_capacity: i32
}

impl InsertStockItem {
    pub async fn insert(
        self,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<StockItem, sqlx::Error> {
        sqlx::query_as!(
            StockItem,
            r#"
            INSERT INTO stock (
                product_id,
                dealership_rif,
                product_cost,
                product_count,
                vendor_name,
                max_capacity,
                min_capacity
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7
            )
            RETURNING
                product_id,
                dealership_rif,
                product_cost,
                product_count,
                vendor_name,
                max_capacity,
                min_capacity
            "#,
            self.product_id,
            self.dealership_rif as _,
            self.product_cost,
            self.product_count,
            self.vendor_name,
            self.max_capacity,
            self.min_capacity,
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateStockItem {
    pub product_id: Option<i32>,
    pub dealership_rif: Option<String>,
    pub product_cost: Option<BigDecimal>,
    pub product_count: Option<i32>,
    pub vendor_name: Option<String>,
    pub max_capacity: Option<i32>,
    pub min_capacity: Option<i32>
}

impl UpdateStockItem {
    pub async fn update(
        self,
        target: StockItem,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<StockItem, sqlx::Error> {
        let new_product_id = self.product_id.unwrap_or(target.product_id);
        let new_dealership_rif = self.dealership_rif.as_ref().unwrap_or(&target.dealership_rif);
        let new_product_cost = self.product_cost.unwrap_or(target.product_cost);
        let new_product_count = self.product_count.unwrap_or(target.product_count);
        let new_vendor_name = self.vendor_name.unwrap_or(target.vendor_name);
        let new_max_capacity = self.max_capacity.unwrap_or(target.max_capacity);
        let new_min_capacity = self.min_capacity.unwrap_or(target.min_capacity);

        sqlx::query_as!(
            StockItem,
            r#"
            UPDATE stock
            SET
                product_id = $1,
                dealership_rif = $2,
                product_cost = $3,
                product_count = $4,
                vendor_name = $5,
                max_capacity = $6,
                min_capacity = $7
            WHERE
                product_id = $8
                AND dealership_rif = $9
            RETURNING
                product_id,
                dealership_rif,
                product_cost,
                product_count,
                vendor_name,
                max_capacity,
                min_capacity
            "#,
            new_product_id,
            new_dealership_rif as _,
            new_product_cost,
            new_product_count,
            new_vendor_name,
            new_max_capacity,
            new_min_capacity,
            target.product_id,
            target.dealership_rif as _
        )
        .fetch_one(connection)
        .await
    }
}
