use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use time::Date;

use crate::utils::pagination::{Page, Pages, Paginable};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vehicle {
    pub plate: String,
    pub brand: String,
    pub model_id: i32,
    pub serial_no: String,
    pub engine_serial_no: String,
    pub color: String,
    pub purchase_date: Date,
    pub additional_info: Option<String>,
    pub maintenance_summary: Option<String>,
    pub owner_national_id: String,
}

impl Vehicle {
    pub async fn select(
        plate: String,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vehicle, sqlx::Error> {
        sqlx::query_as!(
            Vehicle,
            r#"
            SELECT
                plate,
                brand,
                model_id,
                serial_no,
                engine_serial_no,
                color,
                purchase_date,
                additional_info,
                maintenance_summary,
                owner_national_id
            FROM vehicles
            WHERE
                plate = $1
            "#,
            plate
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<Vehicle>, sqlx::Error> {
        sqlx::query_as!(
            Vehicle,
            r#"
            SELECT
                plate,
                brand,
                model_id,
                serial_no,
                engine_serial_no,
                color,
                purchase_date,
                additional_info,
                maintenance_summary,
                owner_national_id
            FROM vehicles
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
            SELECT COUNT(*) AS "total_vehicles!"
            FROM vehicles
            "#
        )
        .fetch_one(connection)
        .await
    }

    pub async fn delete(
        plate: String,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vehicle, sqlx::Error> {
        sqlx::query_as!(
            Vehicle,
            r#"
            DELETE FROM vehicles
            WHERE
                plate = $1
            RETURNING
                plate,
                brand,
                model_id,
                serial_no,
                engine_serial_no,
                color,
                purchase_date,
                additional_info,
                maintenance_summary,
                owner_national_id
            "#,
            plate
        )
        .fetch_one(connection)
        .await
    }
}

#[async_trait]
impl Paginable<Vehicle> for Vehicle {
    async fn get_page(
        pages: &Pages<Vehicle, Vehicle>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<Vehicle>, sqlx::Error> {
        let page_items = sqlx::query_as!(
            Vehicle,
            r#"
                SELECT
                    plate,
                    brand,
                    model_id,
                    serial_no,
                    engine_serial_no,
                    color,
                    purchase_date,
                    additional_info,
                    maintenance_summary,
                    owner_national_id
                FROM vehicles
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
pub struct InsertVehicle {
    pub plate: String,
    pub brand: String,
    pub model_id: i32,
    pub serial_no: String,
    pub engine_serial_no: String,
    pub color: String,
    pub purchase_date: Date,
    pub additional_info: Option<String>,
    pub maintenance_summary: Option<String>,
    pub owner_national_id: String,
}

impl InsertVehicle {
    pub async fn insert(
        self,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vehicle, sqlx::Error> {
        sqlx::query_as!(
            Vehicle,
            r#"
            INSERT INTO vehicles (
                plate,
                brand,
                model_id,
                serial_no,
                engine_serial_no,
                color,
                purchase_date,
                additional_info,
                maintenance_summary,
                owner_national_id
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10
            )
            RETURNING
                plate,
                brand,
                model_id,
                serial_no,
                engine_serial_no,
                color,
                purchase_date,
                additional_info,
                maintenance_summary,
                owner_national_id
            "#,
            self.plate,
            self.brand,
            self.model_id,
            self.serial_no,
            self.engine_serial_no,
            self.color,
            self.purchase_date,
            self.additional_info,
            self.maintenance_summary,
            self.owner_national_id as _,
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateVehicle {
    pub plate: Option<String>,
    pub brand: Option<String>,
    pub model_id: Option<i32>,
    pub serial_no: Option<String>,
    pub engine_serial_no: Option<String>,
    pub color: Option<String>,
    pub purchase_date: Option<Date>,
    pub additional_info: Option<Option<String>>,
    pub maintenance_summary: Option<Option<String>>,
    pub owner_national_id: Option<String>,
}

impl UpdateVehicle {
    pub async fn update(
        self,
        target: Vehicle,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vehicle, sqlx::Error> {
        let new_plate = self.plate.as_ref().unwrap_or(&target.plate);
        let new_brand = self.brand.unwrap_or(target.brand);
        let new_model_id = self.model_id.unwrap_or(target.model_id);
        let new_serial_no = self.serial_no.unwrap_or(target.serial_no);
        let new_engine_serial_no = self.engine_serial_no.unwrap_or(target.engine_serial_no);
        let new_color = self.color.unwrap_or(target.color);
        let new_purchase_date = self.purchase_date.unwrap_or(target.purchase_date);
        let new_additional_info = self.additional_info.unwrap_or(target.additional_info);
        let new_maintenance_summary = self
            .maintenance_summary
            .unwrap_or(target.maintenance_summary);
        let new_owner_national_id = self.owner_national_id.unwrap_or(target.owner_national_id);

        sqlx::query_as!(
            Vehicle,
            r#"
            UPDATE vehicles
            SET
                plate = $1,
                brand = $2,
                model_id = $3,
                serial_no = $4,
                engine_serial_no = $5,
                color = $6,
                purchase_date = $7,
                additional_info = $8,
                maintenance_summary = $9,
                owner_national_id = $10
            WHERE
                plate = $11
            RETURNING
                plate,
                brand,
                model_id,
                serial_no,
                engine_serial_no,
                color,
                purchase_date,
                additional_info,
                maintenance_summary,
                owner_national_id
            "#,
            new_plate,
            new_brand,
            new_model_id,
            new_serial_no,
            new_engine_serial_no,
            new_color,
            new_purchase_date,
            new_additional_info,
            new_maintenance_summary,
            new_owner_national_id as _,
            target.plate
        )
        .fetch_one(connection)
        .await
    }
}
