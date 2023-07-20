use serde::Serialize;
use sqlx::{Executor, Postgres};
use time::PrimitiveDateTime;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleAppliedService {
    pub plate: String,
    pub service_id: i32,
    pub checkin_timestamp: PrimitiveDateTime,
}

impl VehicleAppliedService {
    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<VehicleAppliedService>, sqlx::Error> {
        sqlx::query_as!(
            VehicleAppliedService,
            r#"
            WITH paid_orders AS (
                SELECT
                    o.id,
                    vehicle_plate,
                    checkin_timestamp
                FROM
                    orders AS o
                    INNER JOIN invoices AS i ON o.id = i.order_id
            )
            SELECT
                v.plate,
                od.service_id,
                po.checkin_timestamp AS "checkin_timestamp!"
            FROM
                orders_details AS od
                INNER JOIN paid_orders AS po ON od.order_id = po.id
                INNER JOIN vehicles AS v ON po.vehicle_plate = v.plate
            GROUP BY
                v.plate,
                od.service_id,
                po.checkin_timestamp
            ORDER BY
                v.plate ASC,
                po.checkin_timestamp ASC
            "#
        )
        .fetch_all(connection)
        .await
    }
}
