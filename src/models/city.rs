use serde::{Serialize, Deserialize};
use sqlx::{
    Executor,
    Postgres
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct City {
    #[serde(skip_deserializing)]
    pub city_number: i32,
    pub name: String,
    pub state_id: i32
}

pub async fn select_all(state_id: i32, connection: impl Executor<'_, Database = Postgres>) -> Result<Vec<City>, sqlx::Error> {
    sqlx::query_as!(
        City,
        r#"
        SELECT id, name, state_id
        FROM cities
        "#
    )
    .fetch_all(connection)
    .await
}

pub async fn select(city_id: i32, connection: impl Executor<'_, Database = Postgres>) -> Result<City, sqlx::Error> {
    sqlx::query_as!(
        City,
        r#"
        SELECT id, name, state_id
        FROM cities
        WHERE id = $1
        "#,
        city_id
    )
    .fetch_one(connection)
    .await
}

pub async fn insert(city: &City, connection: impl Executor<'_, Database = Postgres>) -> Result<City, sqlx::Error> {
    sqlx::query_as!(
        City,
        r#"
        INSERT INTO cities (name, state_id)
        VALUES ($1, $2)
        RETURNING id, name, state_id
        "#,
        city.name,
        city.state_id
    )
    .fetch_one(connection)
    .await
}

pub async fn update(city: &City, connection: impl Executor<'_, Database = Postgres>) -> Result<City, sqlx::Error> {
    sqlx::query_as!(
        City,
        r#"
        UPDATE cities
        SET name = $1, state_id = $2
        WHERE id = $3
        RETURNING id, name, state_id
        "#,
        city.name,
        city.state_id,
        city.id
    )
    .fetch_one(connection)
    .await
}

pub async fn delete(city_id: i32, connection: impl Executor<'_, Database = Postgres>) -> Result<City, sqlx::Error> {
    sqlx::query_as!(
        City,
        r#"
        DELETE FROM cities
        WHERE id = $1
        RETURNING id, name, state_id
        "#,
        city_id
    )
    .fetch_one(connection)
    .await
}