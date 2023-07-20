use serde::Serialize;
use sqlx::{Executor, Postgres};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoShowClient {
    pub national_id: String,
    pub full_name: String,
}

impl NoShowClient {
    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<NoShowClient>, sqlx::Error> {
        sqlx::query_as!(
            NoShowClient,
            r#"
            SELECT
                c.national_id,
                c.full_name
            FROM
                invoices AS i
                RIGHT JOIN orders AS o ON i.order_id = o.id
                INNER JOIN vehicles AS v ON o.vehicle_plate = v.plate
                INNER JOIN clients AS c ON v.owner_national_id = c.national_id
            WHERE
                i.id IS NULL
            "#
        )
        .fetch_all(connection)
        .await
    }
}
