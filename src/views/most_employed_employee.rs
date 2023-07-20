use serde::Serialize;
use sqlx::{Executor, Postgres};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MostEmployedEmployee {
    pub national_id: String,
    pub full_name: String,
    pub realized_services_count: i64,
}

impl MostEmployedEmployee {
    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<MostEmployedEmployee>, sqlx::Error> {
        sqlx::query_as!(
            MostEmployedEmployee,
            r#"
            WITH paid_orders AS (
                SELECT
                    o.id,
                    o.vehicle_plate
                FROM
                    orders AS o
                    INNER JOIN invoices AS i ON o.id = i.order_id
            ),
            realized_services_count_per_employee AS (
                SELECT
                    s.national_id,
                    s.full_name,
                    COUNT(*) AS realized_services_count
                FROM
                    paid_orders AS po
                    INNER JOIN orders_details AS od ON po.id = od.order_id
                    INNER JOIN products_applications AS pa ON od.service_id = pa.service_id
                    INNER JOIN staff AS s ON pa.employee_national_id = s.national_id
                GROUP BY
                    s.national_id,
                    s.full_name
            )
            SELECT
                national_id,
                full_name,
                realized_services_count AS "realized_services_count!"
            FROM
                realized_services_count_per_employee AS rscpe
            WHERE
                rscpe.realized_services_count = (SELECT MAX(realized_services_count) FROM realized_services_count_per_employee)
            "#,
        )
        .fetch_all(connection)
        .await
    }
}
