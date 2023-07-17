use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};

use crate::utils::pagination::{Page, Pages, Paginable};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplyLine {
    pub id: i32,
    pub name: String,
}

impl SupplyLine {
    pub async fn select(
        id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<SupplyLine, sqlx::Error> {
        sqlx::query_as!(
            SupplyLine,
            r#"
            SELECT id, name
            FROM supply_lines
            WHERE id = $1
            "#,
            id,
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<SupplyLine>, sqlx::Error> {
        sqlx::query_as!(
            SupplyLine,
            r#"
            SELECT id, name
            FROM supply_lines
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
            SELECT COUNT(*) AS "total_supply_lines!"
            FROM supply_lines
            "#
        )
        .fetch_one(connection)
        .await
    }

    pub async fn delete(
        id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<SupplyLine, sqlx::Error> {
        sqlx::query_as!(
            SupplyLine,
            r#"
            DELETE FROM supply_lines
            WHERE id = $1
            RETURNING id, name
            "#,
            id,
        )
        .fetch_one(connection)
        .await
    }
}

#[async_trait]
impl Paginable<SupplyLine> for SupplyLine {
    async fn get_page(
        pages: &Pages<SupplyLine, SupplyLine>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<SupplyLine>, sqlx::Error> {
        let page_items = sqlx::query_as!(
            SupplyLine,
            r#"
                SELECT id, name
                FROM supply_lines
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
pub struct InsertSupplyLine {
    pub name: String,
}

impl InsertSupplyLine {
    pub async fn insert(
        self,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<SupplyLine, sqlx::Error> {
        sqlx::query_as!(
            SupplyLine,
            r#"
            INSERT INTO supply_lines (name)
            VALUES ($1)
            RETURNING id, name
            "#,
            self.name,
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateSupplyLine {
    pub name: Option<String>,
}

impl UpdateSupplyLine {
    pub async fn update(
        self,
        target: SupplyLine,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<SupplyLine, sqlx::Error> {
        let new_name = self.name.unwrap_or(target.name);

        sqlx::query_as!(
            SupplyLine,
            r#"
            UPDATE supply_lines
            SET name = $1
            WHERE id = $2
            RETURNING id, name
            "#,
            new_name,
            target.id
        )
        .fetch_one(connection)
        .await
    }
}
