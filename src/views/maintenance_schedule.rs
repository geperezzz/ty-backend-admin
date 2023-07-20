use bigdecimal::BigDecimal;
use serde::Serialize;
use sqlx::{Executor, Postgres};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceSchedule {
    pub vehicle_model_id: i32,
    pub vehicle_model_name: String,
    pub service_id: i32,
    pub service_name: String,
    pub required_usage_time: String,
    pub required_kilometrage: BigDecimal,
}

impl MaintenanceSchedule {
    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<MaintenanceSchedule>, sqlx::Error> {
        sqlx::query_as!(
            MaintenanceSchedule,
            r#"
            SELECT
                vm.id AS vehicle_model_id,
                vm.name AS vehicle_model_name,
                s.id AS service_id,
                s.name AS service_name,
                TO_CHAR(rs.required_usage_time, 'FMYYYY año/s y FMMM mes/es') AS "required_usage_time!",
                rs.required_kilometrage
            FROM
                services AS s
                INNER JOIN recommended_services AS rs ON s.id = rs.service_id
                INNER JOIN vehicle_models AS vm ON rs.vehicle_model_id = vm.id
            "#
        )
        .fetch_all(connection)
        .await
    }
}
