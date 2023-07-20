use serde::Serialize;
use sqlx::{Executor, Postgres};
use time::Date;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MostAttendedVehicleModel {
    pub vehicle_model_id: i32,
    pub vehicle_model_name: String,
    pub attendance_count: i64,
}

impl MostAttendedVehicleModel {
    pub async fn select_all_in_range(
        from_date: Date,
        to_date: Date,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<MostAttendedVehicleModel>, sqlx::Error> {
        sqlx::query_as!(
            MostAttendedVehicleModel,
            r#"
            WITH invoices_in_range AS (
                SELECT
                    id,
                    order_id
                FROM
                    invoices
                WHERE
                    issue_date BETWEEN $1 AND $2
            ),
            attendances AS (
                SELECT
                    vm.id AS vehicle_model_id,
                    vm.name AS vehicle_model_name,
                    COUNT(*) AS attendance_count
                FROM
                    invoices_in_range AS i
                    INNER JOIN orders AS o ON i.order_id = o.id
                    INNER JOIN vehicles AS v ON o.vehicle_plate = v.plate
                    INNER JOIN vehicle_models AS vm ON v.model_id = vm.id
                GROUP BY
                    vm.id
            )
            SELECT
                vehicle_model_id,
                vehicle_model_name,
                attendance_count AS "attendance_count!"
            FROM
                attendances
            WHERE
                attendance_count = (SELECT MAX(attendance_count) FROM attendances)
            "#,
            from_date,
            to_date
        )
        .fetch_all(connection)
        .await
    }

    pub async fn select_all_by_name(
        name: String,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<MostAttendedVehicleModel>, sqlx::Error> {
        sqlx::query_as!(
            MostAttendedVehicleModel,
            r#"
            WITH paid_orders AS (
                SELECT
                    o.id,
                    vehicle_plate
                FROM
                    orders AS o
                    INNER JOIN invoices AS i ON o.id = i.order_id
            ),
            attendances AS (
                SELECT
                    vm.id AS vehicle_model_id,
                    vm.name AS vehicle_model_name,
                    COUNT(*) AS attendance_count
                FROM
                    paid_orders AS po
                    INNER JOIN orders_details AS od ON po.id = od.order_id
                    INNER JOIN activities AS a ON od.activity_number = a.activity_number
                    INNER JOIN services AS s ON a.service_id = s.id
                    INNER JOIN vehicles AS v ON po.vehicle_plate = v.plate
                    INNER JOIN vehicle_models AS vm ON v.model_id = vm.id
                WHERE
                    s.name = $1
                GROUP BY
                    vm.id
            )
            SELECT
                vehicle_model_id,
                vehicle_model_name,
                attendance_count AS "attendance_count!"
            FROM
                attendances
            WHERE
                attendance_count = (SELECT MAX(attendance_count) FROM attendances);
            "#,
            name
        )
        .fetch_all(connection)
        .await
    }
}
