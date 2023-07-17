use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};

use crate::utils::pagination::{Page, Pages, Paginable};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: String,
}

impl Role {
    pub async fn select(
        id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Role, sqlx::Error> {
        sqlx::query_as!(
            Role,
            r#"
            SELECT id, name, description
            FROM roles
            WHERE id = $1
            "#,
            id,
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<Role>, sqlx::Error> {
        sqlx::query_as!(
            Role,
            r#"
            SELECT id, name, description
            FROM roles
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
            SELECT COUNT(*) AS "total_roles!"
            FROM roles
            "#
        )
        .fetch_one(connection)
        .await
    }

    pub async fn delete(
        id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Role, sqlx::Error> {
        sqlx::query_as!(
            Role,
            r#"
            DELETE FROM roles
            WHERE id = $1
            RETURNING id, name, description
            "#,
            id,
        )
        .fetch_one(connection)
        .await
    }
}

#[async_trait]
impl Paginable<Role> for Role {
    async fn get_page(
        pages: &Pages<Role, Role>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<Role>, sqlx::Error> {
        let page_items = sqlx::query_as!(
            Role,
            r#"
                SELECT id, name, description
                FROM roles
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
pub struct InsertRole {
    pub name: String,
    pub description: String,
}

impl InsertRole {
    pub async fn insert(
        self,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Role, sqlx::Error> {
        sqlx::query_as!(
            Role,
            r#"
            INSERT INTO roles (name, description)
            VALUES ($1, $2)
            RETURNING id, name, description
            "#,
            self.name,
            self.description
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateRole {
    pub name: Option<String>,
    pub description: Option<String>,
}

impl UpdateRole {
    pub async fn update(
        self,
        target: Role,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Role, sqlx::Error> {
        let new_name = self.name.unwrap_or(target.name);
        let new_description = self.description.unwrap_or(target.description);

        sqlx::query_as!(
            Role,
            r#"
            UPDATE roles
            SET
                name = $1,
                description = $2
            WHERE id = $3
            RETURNING id, name, description
            "#,
            new_name,
            new_description,
            target.id
        )
        .fetch_one(connection)
        .await
    }
}
