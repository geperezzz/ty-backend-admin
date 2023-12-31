use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};

use crate::utils::pagination::{Page, Pages, Paginable};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dealership {
    pub rif: String,
    pub name: String,
    pub city_number: i32,
    pub state_id: i32
}

impl Dealership {
    pub async fn select(
        rif: String,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Dealership, sqlx::Error> {
        sqlx::query_as!(
            Dealership,
            r#"
            SELECT 
                rif,
                name,
                city_number,
                state_id
            FROM 
                dealerships
            WHERE
                rif = $1
            "#,
            rif,
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<Dealership>, sqlx::Error> {
        sqlx::query_as!(
            Dealership,
            r#"
            SELECT
                rif,
                name,
                city_number,
                state_id
            FROM 
                dealerships
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
                COUNT(*) AS "total_dealerships!"
            FROM 
                dealerships
            "#
        )
        .fetch_one(connection)
        .await
    }

    pub async fn delete(
        rif: String,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Dealership, sqlx::Error> {
        sqlx::query_as!(
            Dealership,
            r#"
            DELETE FROM dealerships
            WHERE 
                rif = $1
            RETURNING 
                rif,
                name,
                city_number,
                state_id
            "#,
            rif,
        )
        .fetch_one(connection)
        .await
    }
}

#[async_trait]
impl Paginable<Dealership> for Dealership {
    async fn get_page(
        pages: &Pages<Dealership, Dealership>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<Dealership>, sqlx::Error> {
        let page_items = sqlx::query_as!(
            Dealership,
            r#"
                SELECT 
                    rif,
                    name,
                    city_number,
                    state_id
                FROM 
                    dealerships
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
pub struct InsertDealership {
    pub rif: String,
    pub name: String,
    pub city_number: i32,
    pub state_id: i32,
}

impl InsertDealership {
    pub async fn insert(
        self,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Dealership, sqlx::Error> {
        sqlx::query_as!(
            Dealership,
            r#"
            INSERT INTO dealerships 
                (rif, name, city_number, state_id)
            VALUES 
                ($1, $2, $3, $4)
            RETURNING 
                rif,
                name,
                city_number,
                state_id
            "#,
            self.rif as _,
            self.name,
            self.city_number,
            self.state_id,
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateDealership {
    pub rif: Option<String>,
    pub name: Option<String>,
    pub city_number: Option<i32>,
    pub state_id: Option<i32>,
}

impl UpdateDealership {
    pub async fn update(
        self,
        target: Dealership,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Dealership, sqlx::Error> {
        let new_rif = self.rif.as_ref().unwrap_or(&target.rif);
        let new_name = self.name.unwrap_or(target.name);
        let new_city_number = self.city_number.unwrap_or(target.city_number);
        let new_state_id = self.state_id.unwrap_or(target.state_id);

        sqlx::query_as!(
            Dealership,
            r#"
            UPDATE dealerships
            SET 
                rif = $1,
                name = $2,
                city_number = $3,
                state_id = $4
            WHERE 
                rif = $5
            RETURNING 
                rif,
                name,
                city_number,
                state_id
            "#,
            new_rif as _,
            new_name,
            new_city_number,
            new_state_id,
            target.rif,
        )
        .fetch_one(connection)
        .await
    }
}
