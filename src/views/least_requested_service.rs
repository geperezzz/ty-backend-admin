use serde::Serialize;
use sqlx::{Executor, Postgres};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeastRequestedService {
    pub id: i32,
    pub name: String,
    pub count: i64,
}

impl LeastRequestedService {
    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<LeastRequestedService>, sqlx::Error> {
        sqlx::query_as!(
            LeastRequestedService,
            r#"
            WITH requests_count_per_service AS (
                SELECT
                    s.id,
                    s.name,
                    COUNT(*) AS count 
                FROM
                    orders_details AS od
                    INNER JOIN activities AS a ON od.activity_number = a.activity_number
                    INNER JOIN services AS s ON a.service_id = s.id
                GROUP BY
                    s.id,
                    s.name
            )
            SELECT
                id,
                name,
                count AS "count!"
            FROM 
                requests_count_per_service AS rcps
            WHERE
                rcps.count = (SELECT MIN(count) FROM requests_count_per_service)            
            "#
        )
        .fetch_all(connection)
        .await
    }
}
