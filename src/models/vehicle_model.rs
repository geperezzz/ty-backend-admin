use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use bigdecimal::BigDecimal;

use crate::utils::pagination::{Page, Pages, Paginable};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleModel {
    pub id: i32,
    pub name: String,
    pub seat_count: i32,
    pub weight_in_kg: BigDecimal,
    pub octane_rating: i16,
    pub gearbox_oil_type: String,
    pub engine_oil_type: String,
    pub engine_coolant_type: String,
}

impl VehicleModel {
    pub async fn select(
        id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<VehicleModel, sqlx::Error> {
        sqlx::query_as!(
            VehicleModel,
            r#"
            SELECT
                id,
                name,
                seat_count,
                weight_in_kg,
                octane_rating,
                gearbox_oil_type,
                engine_oil_type,
                engine_coolant_type
            FROM vehicle_models
            WHERE
                id = $1
            "#,
            id
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<VehicleModel>, sqlx::Error> {
        sqlx::query_as!(
            VehicleModel,
            r#"
            SELECT
                id,
                name,
                seat_count,
                weight_in_kg,
                octane_rating,
                gearbox_oil_type,
                engine_oil_type,
                engine_coolant_type
            FROM vehicle_models
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
            SELECT COUNT(*) AS "total_vehicle_models!"
            FROM vehicle_models
            "#
        )
        .fetch_one(connection)
        .await
    }

    pub async fn delete(
        id: i32,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<VehicleModel, sqlx::Error> {
        sqlx::query_as!(
            VehicleModel,
            r#"
            DELETE FROM vehicle_models
            WHERE
                id = $1
            RETURNING
                id,
                name,
                seat_count,
                weight_in_kg,
                octane_rating,
                gearbox_oil_type,
                engine_oil_type,
                engine_coolant_type
            "#,
            id
        )
        .fetch_one(connection)
        .await
    }
}

#[async_trait]
impl Paginable<VehicleModel> for VehicleModel {
    async fn get_page(
        pages: &Pages<VehicleModel, VehicleModel>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<VehicleModel>, sqlx::Error> {
        let page_items = sqlx::query_as!(
            VehicleModel,
            r#"
            SELECT
                id,
                name,
                seat_count,
                weight_in_kg,
                octane_rating,
                gearbox_oil_type,
                engine_oil_type,
                engine_coolant_type
            FROM vehicle_models
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
pub struct InsertVehicleModel {
    pub name: String,
    pub seat_count: i32,
    pub weight_in_kg: BigDecimal,
    pub octane_rating: i16,
    pub gearbox_oil_type: String,
    pub engine_oil_type: String,
    pub engine_coolant_type: String,
}

impl InsertVehicleModel {
    pub async fn insert(
        self,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<VehicleModel, sqlx::Error> {
        sqlx::query_as!(
            VehicleModel,
            r#"
            INSERT INTO vehicle_models (
                name,
                seat_count,
                weight_in_kg,
                octane_rating,
                gearbox_oil_type,
                engine_oil_type,
                engine_coolant_type
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7
            )
            RETURNING
                id,
                name,
                seat_count,
                weight_in_kg,
                octane_rating,
                gearbox_oil_type,
                engine_oil_type,
                engine_coolant_type
            "#,
            self.name,
            self.seat_count,
            self.weight_in_kg,
            self.octane_rating,
            self.gearbox_oil_type,
            self.engine_oil_type,
            self.engine_coolant_type
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateVehicleModel {
    pub name: Option<String>,
    pub seat_count: Option<i32>,
    pub weight_in_kg: Option<BigDecimal>,
    pub octane_rating: Option<i16>,
    pub gearbox_oil_type: Option<String>,
    pub engine_oil_type: Option<String>,
    pub engine_coolant_type: Option<String>,
}

impl UpdateVehicleModel {
    pub async fn update(
        self,
        target: VehicleModel,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<VehicleModel, sqlx::Error> {
        let new_name = self.name.unwrap_or(target.name);
        let new_seat_count = self.seat_count.unwrap_or(target.seat_count);
        let new_weight_in_kg = self.weight_in_kg.unwrap_or(target.weight_in_kg);
        let new_octane_rating = self.octane_rating.unwrap_or(target.octane_rating);
        let new_gearbox_oil_type = self.gearbox_oil_type.unwrap_or(target.gearbox_oil_type);
        let new_engine_oil_type = self.engine_oil_type.unwrap_or(target.engine_oil_type);
        let new_engine_coolat_type = self.engine_coolant_type.unwrap_or(target.engine_coolant_type);

        sqlx::query_as!(
            VehicleModel,
            r#"
            UPDATE vehicle_models
            SET
                name = $1,
                seat_count = $2,
                weight_in_kg = $3,
                octane_rating = $4,
                gearbox_oil_type = $5,
                engine_oil_type = $6,
                engine_coolant_type = $7
            WHERE
                id = $8
            RETURNING
                id,
                name,
                seat_count,
                weight_in_kg,
                octane_rating,
                gearbox_oil_type,
                engine_oil_type,
                engine_coolant_type
            "#,
            new_name as _,
            new_seat_count,
            new_weight_in_kg,
            new_octane_rating,
            new_gearbox_oil_type,
            new_engine_oil_type,
            new_engine_coolat_type,
            target.id as _
        )
        .fetch_one(connection)
        .await
    }
}