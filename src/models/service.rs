use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};

use crate::utils::pagination::{Page, Pages, Paginable};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub coordinator_national_id: String,
}

impl Service {
    pub async fn select(
        id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Service, sqlx::Error> {
        sqlx::query_as!(
            Service,
            r#"
            SELECT 
                id,
                name,
                description,
                coordinator_national_id
            FROM 
                services
            WHERE 
                id = $1
            "#,
            id,
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<Service>, sqlx::Error> {
        sqlx::query_as!(
            Service,
            r#"
            SELECT 
                id,
                name,
                description,
                coordinator_national_id
            FROM 
                services
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
            SELECT COUNT(*) AS "total_services!"
            FROM services
            "#
        )
        .fetch_one(connection)
        .await
    }

    pub async fn delete(
        id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Service, sqlx::Error> {
        sqlx::query_as!(
            Service,
            r#"
            DELETE FROM services
            WHERE id = $1
            RETURNING
                id,
                name,
                description,
                coordinator_national_id
            "#,
            id,
        )
        .fetch_one(connection)
        .await
    }
}

#[async_trait]
impl Paginable<Service> for Service {
    async fn get_page(
        pages: &Pages<Service, Service>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<Service>, sqlx::Error> {
        let page_items = sqlx::query_as!(
            Service,
            r#"
                SELECT 
                    id,
                    name,
                    description,
                    coordinator_national_id
                FROM 
                    services
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
pub struct InsertService {
    pub name: String,
    pub description: String,
    pub coordinator_national_id: String,
}

impl InsertService {
    pub async fn insert(
        self,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Service, sqlx::Error> {
        sqlx::query_as!(
            Service,
            r#"
            INSERT INTO services 
                (name, description, coordinator_national_id)
            VALUES 
                ($1, $2, $3)
            RETURNING 
                id,
                name,
                description,
                coordinator_national_id
            "#,
            self.name,
            self.description,
            self.coordinator_national_id as _
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateService {
    pub name: Option<String>,
    pub description: Option<String>,
    pub coordinator_national_id: Option<String>,
}

impl UpdateService {
    pub async fn update(
        self,
        target: Service,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Service, sqlx::Error> {
        let new_name = self.name.as_ref().unwrap_or(&target.name);
        let new_description = self.description.as_ref().unwrap_or(&target.description);
        let new_coordinator_national_id = self
            .coordinator_national_id
            .as_ref()
            .unwrap_or(&target.coordinator_national_id);

        sqlx::query_as!(
            Service,
            r#"
            UPDATE services
            SET 
                name = $1,
                description = $2,
                coordinator_national_id = $3
            WHERE id = $4
            RETURNING 
                id,
                name,
                description,
                coordinator_national_id
            "#,
            new_name,
            new_description,
            new_coordinator_national_id as _,
            target.id
        )
        .fetch_one(connection)
        .await
    }
}
