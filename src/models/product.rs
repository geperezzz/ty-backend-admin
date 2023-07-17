use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};

use crate::utils::pagination::{Page, Pages, Paginable};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub is_ecologic: bool,
    pub supply_line_id: i32,
}

impl Product {
    pub async fn select(
        id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Product, sqlx::Error> {
        sqlx::query_as!(
            Product,
            r#"
            SELECT
                id,
                name,
                description,
                is_ecologic,
                supply_line_id
            FROM products
            WHERE id = $1
            "#,
            id,
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<Product>, sqlx::Error> {
        sqlx::query_as!(
            Product,
            r#"
            SELECT
                id,
                name,
                description,
                is_ecologic,
                supply_line_id
            FROM products
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
            SELECT COUNT(*) AS "total_products!"
            FROM products
            "#
        )
        .fetch_one(connection)
        .await
    }

    pub async fn delete(
        id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Product, sqlx::Error> {
        sqlx::query_as!(
            Product,
            r#"
            DELETE FROM products
            WHERE id = $1
            RETURNING
                id,
                name,
                description,
                is_ecologic,
                supply_line_id
            "#,
            id,
        )
        .fetch_one(connection)
        .await
    }
}

#[async_trait]
impl Paginable<Product> for Product {
    async fn get_page(
        pages: &Pages<Product, Product>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<Product>, sqlx::Error> {
        let page_items = sqlx::query_as!(
            Product,
            r#"
                SELECT
                    id,
                    name,
                    description,
                    is_ecologic,
                    supply_line_id
                FROM products
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
pub struct InsertProduct {
    pub name: String,
    pub description: String,
    pub is_ecologic: bool,
    pub supply_line_id: i32,
}

impl InsertProduct {
    pub async fn insert(
        self,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Product, sqlx::Error> {
        sqlx::query_as!(
            Product,
            r#"
            INSERT INTO products (
                name,
                description,
                is_ecologic,
                supply_line_id
            )
            VALUES (
                $1,
                $2,
                $3,
                $4
            )
            RETURNING
                id,
                name,
                description,
                is_ecologic,
                supply_line_id
            "#,
            self.name,
            self.description,
            self.is_ecologic,
            self.supply_line_id,
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateProduct {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_ecologic: Option<bool>,
    pub supply_line_id: Option<i32>,
}

impl UpdateProduct {
    pub async fn update(
        self,
        target: Product,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Product, sqlx::Error> {
        let new_name = self.name.unwrap_or(target.name);
        let new_description = self.description.unwrap_or(target.description);
        let new_is_ecologic = self.is_ecologic.unwrap_or(target.is_ecologic);
        let new_supply_line_id = self.supply_line_id.unwrap_or(target.supply_line_id);

        sqlx::query_as!(
            Product,
            r#"
            UPDATE products
            SET
                name = $1,
                description = $2,
                is_ecologic = $3,
                supply_line_id = $4
            WHERE id = $5
            RETURNING
                id,
                name,
                description,
                is_ecologic,
                supply_line_id
            "#,
            new_name,
            new_description,
            new_is_ecologic,
            new_supply_line_id,
            target.id,
        )
        .fetch_one(connection)
        .await
    }
}
