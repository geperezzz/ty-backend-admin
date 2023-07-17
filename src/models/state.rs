use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};

use crate::utils::pagination::{Page, Pages, Paginable};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub id: i32,
    pub name: String
}

impl State {
    pub async fn select(
        id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<State, sqlx::Error> {
        sqlx::query_as!(
            State,
            r#"
            SELECT
                id,
                name
            FROM states
            WHERE
                id = $1
            "#,
            id
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<State>, sqlx::Error> {
        sqlx::query_as!(
            State,
            r#"
            SELECT
                id,
                name
            FROM states
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
            SELECT COUNT(*) AS "total_states!"
            FROM states
            "#
        )
        .fetch_one(connection)
        .await
    }

    pub async fn delete(
        id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<State, sqlx::Error> {
        sqlx::query_as!(
            State,
            r#"
            DELETE FROM states
            WHERE
                id = $1
            RETURNING
                id,
                name
            "#,
            id
        )
        .fetch_one(connection)
        .await
    }
}

#[async_trait]
impl Paginable<State> for State {
    async fn get_page(
        pages: &Pages<State, State>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<State>, sqlx::Error> {
        let page_items = sqlx::query_as!(
            State,
            r#"
                SELECT
                    id,
                    name
                FROM states
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
pub struct InsertState {
    pub name: String
}

impl InsertState {
    pub async fn insert(
        self,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<State, sqlx::Error> {
        sqlx::query_as!(
            State,
            r#"
            INSERT INTO states (
                name
            )
            VALUES (
                $1
            )
            RETURNING
                id,
                name
            "#,
            self.name as _
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateState {
    pub name: Option<String>
}

impl UpdateState {
    pub async fn update(
        self,
        target: State,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<State, sqlx::Error> {
        let new_name = self.name.unwrap_or(target.name);

        sqlx::query_as!(
            State,
            r#"
            UPDATE states
            SET
                name = $1
            WHERE
                id = $2
            RETURNING
                id,
                name
            "#,
            new_name as _,
            target.id as _
        )
        .fetch_one(connection)
        .await
    }
}