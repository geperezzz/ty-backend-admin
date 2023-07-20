use serde::Serialize;
use sqlx::{Executor, Postgres};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeastUsedProduct {
    pub id: i32,
    pub name: String,
    pub count: i64,
}

impl LeastUsedProduct {
    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<LeastUsedProduct>, sqlx::Error> {
        sqlx::query_as!(
            LeastUsedProduct,
            r#"
            WITH paid_orders AS (
                SELECT
                    o.id
                FROM
                    orders AS o
                    INNER JOIN invoices AS i ON o.id = i.order_id
            ),
            product_usage_count AS (
                SELECT
                    p.id,
                    p.name,
                   (COUNT(*) * pa.application_count) AS count
                FROM
                    paid_orders AS po
                    INNER JOIN products_applications AS pa ON po.id = pa.service_id
                    INNER JOIN products AS p ON pa.product_id = p.id
                GROUP BY
                    p.id,
                    p.name,
                    pa.application_count
            )
            SELECT
                id,
                name,
                count AS "count!"
            FROM
                product_usage_count AS puc
            WHERE
                puc.count = (SELECT MIN(count) FROM product_usage_count)
            "#
        )
        .fetch_all(connection)
        .await
    }
}
