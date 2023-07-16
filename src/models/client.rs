use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};

use crate::utils::pagination::{Page, Pages, Paginable};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    pub national_id: String,
    pub full_name: String,
    pub main_phone_no: String,
    pub secondary_phone_no: String,
    pub email: String,
}

impl Client {
    pub async fn select(
        national_id: String,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Client, sqlx::Error> {
        sqlx::query_as!(
            Client,
            r#"
            SELECT
                national_id,
                full_name,
                main_phone_no,
                secondary_phone_no,
                email
            FROM clients
            WHERE
                national_id = $1
            "#,
            national_id
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<Client>, sqlx::Error> {
        sqlx::query_as!(
            Client,
            r#"
            SELECT
                national_id,
                full_name,
                main_phone_no,
                secondary_phone_no,
                email
            FROM clients
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
            SELECT COUNT(*) AS "total_clients!"
            FROM clients
            "#
        )
        .fetch_one(connection)
        .await
    }

    pub async fn delete(
        national_id: String,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Client, sqlx::Error> {
        sqlx::query_as!(
            Client,
            r#"
            DELETE FROM clients
            WHERE
                national_id = $1
            RETURNING
                national_id,
                full_name,
                main_phone_no,
                secondary_phone_no,
                email
            "#,
            national_id
        )
        .fetch_one(connection)
        .await
    }
}

#[async_trait]
impl Paginable<Client> for Client {
    async fn get_page(
        pages: &Pages<Client, Client>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<Client>, sqlx::Error> {
        let page_items = sqlx::query_as!(
            Client,
            r#"
                SELECT
                    national_id,
                    full_name,
                    main_phone_no,
                    secondary_phone_no,
                    email
                FROM clients
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
pub struct InsertClient {
    pub national_id: String,
    pub full_name: String,
    pub main_phone_no: String,
    pub secondary_phone_no: String,
    pub email: String,
}

impl InsertClient {
    pub async fn insert(
        self,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Client, sqlx::Error> {
        sqlx::query_as!(
            Client,
            r#"
            INSERT INTO clients (
                national_id,
                full_name,
                main_phone_no,
                secondary_phone_no,
                email
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5
            )
            RETURNING
                national_id,
                full_name,
                main_phone_no,
                secondary_phone_no,
                email
            "#,
            self.national_id as _,
            self.full_name,
            self.main_phone_no as _,
            self.secondary_phone_no as _,
            self.email as _
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateClient {
    pub national_id: Option<String>,
    pub full_name: Option<String>,
    pub main_phone_no: Option<String>,
    pub secondary_phone_no: Option<String>,
    pub email: Option<String>,
}

impl UpdateClient {
    pub async fn update(
        self,
        target: Client,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Client, sqlx::Error> {
        let new_national_id = self.national_id.as_ref().unwrap_or(&target.national_id);
        let new_full_name = self.full_name.unwrap_or(target.full_name);
        let new_main_phone_no = self.main_phone_no.unwrap_or(target.main_phone_no);
        let new_secondary_phone_no = self.secondary_phone_no.unwrap_or(target.secondary_phone_no);
        let new_email = self.email.unwrap_or(target.email);

        sqlx::query_as!(
            Client,
            r#"
            UPDATE clients
            SET
                national_id = $1,
                full_name = $2,
                main_phone_no = $3,
                secondary_phone_no = $4,
                email = $5
            WHERE
                national_id = $6
            RETURNING
                national_id,
                full_name,
                main_phone_no,
                secondary_phone_no,
                email
            "#,
            new_national_id as _,
            new_full_name,
            new_main_phone_no as _,
            new_secondary_phone_no as _,
            new_email as _,
            target.national_id as _
        )
        .fetch_one(connection)
        .await
    }
}
