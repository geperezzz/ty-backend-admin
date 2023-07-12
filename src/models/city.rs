use sqlx::{
    Executor,
    Postgres
};
use serde::{
    Serialize,
    Deserialize
};

#[derive(Serialize, Deserialize)]
pub struct City {
    pub city_number: i32,
    pub name: String,
    pub state_id: i32
}

impl City {
    pub async fn select(city_number: i32, state_id: i32, connection: impl Executor<'_, Database = Postgres>) -> Result<City, sqlx::Error> {
        sqlx::query_as!(
            City,
            r#"
            SELECT city_number, name, state_id
            FROM cities
            WHERE
                city_number = $1
                AND state_id = $2
            "#,
            city_number,
            state_id
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(connection: impl Executor<'_, Database = Postgres>) -> Result<Vec<City>, sqlx::Error> {
        sqlx::query_as!(
            City,
            r#"
            SELECT city_number, name, state_id
            FROM cities
            "#
        )
        .fetch_all(connection)
        .await
    }

    pub async fn delete(city_number: i32, state_id: i32, connection: impl Executor<'_, Database = Postgres>) -> Result<City, sqlx::Error> {
        sqlx::query_as!(
            City,
            r#"
            DELETE FROM cities
            WHERE
                city_number = $1
                AND state_id = $2
            RETURNING city_number, name, state_id
            "#,
            city_number,
            state_id
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct InsertCity {
    pub name: String,
    pub state_id: i32
}

impl InsertCity {
    pub async fn insert(self, connection: impl Executor<'_, Database = Postgres>) -> Result<City, sqlx::Error> {
        sqlx::query_as!(
            City,
            r#"
            INSERT INTO cities (name, state_id)
            VALUES ($1, $2)
            RETURNING city_number, name, state_id
            "#,
            self.name,
            self.state_id
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateCity {
    pub name: Option<String>,
    pub state_id: Option<i32>
}

impl UpdateCity {
    pub async fn update(self, target: City, connection: impl Executor<'_, Database = Postgres>) -> Result<City, sqlx::Error> {
        let new_name = self.name.unwrap_or(target.name);
        let new_state_id = self.state_id.unwrap_or(target.state_id);

        sqlx::query_as!(
            City,
            r#"
            UPDATE cities
            SET
                name = $1,
                state_id = $2
            WHERE
                city_number = $3
                AND state_id = $4
            RETURNING city_number, name, state_id
            "#,
            new_name,
            new_state_id,
            target.city_number,
            target.state_id
        )
        .fetch_one(connection)
        .await
    }
}