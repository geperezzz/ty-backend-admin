use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use bigdecimal::BigDecimal;

use crate::utils::pagination::{Page, Pages, Paginable};

use super::role;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Employee {
    pub national_id: String,
    pub full_name: String,
    pub main_phone_no: String,
    pub secondary_phone_no: String,
    pub email: String,
    pub address: String,
    pub role_id: i32,
    pub salary: BigDecimal,
}

impl Employee {
    pub async fn select(
        national_id: String,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Employee, sqlx::Error> {
        sqlx::query_as!(
            Employee,
            r#"
            SELECT
                national_id,
                full_name,
                main_phone_no,
                secondary_phone_no,
                email,
                address,
                role_id,
                salary
            FROM staff
            WHERE national_id = $1
            "#,
            national_id,
        )
        .fetch_one(connection)
        .await
    }

    pub async fn select_all(
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<Employee>, sqlx::Error> {
        sqlx::query_as!(
            Employee,
            r#"
            SELECT
                national_id,
                full_name,
                main_phone_no,
                secondary_phone_no,
                email,
                address,
                role_id,
                salary
            FROM staff
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
            SELECT COUNT(*) AS "total_staff!"
            FROM staff
            "#
        )
        .fetch_one(connection)
        .await
    }

    pub async fn delete(
        national_id: String,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Employee, sqlx::Error> {
        sqlx::query_as!(
            Employee,
            r#"
            DELETE FROM staff
            WHERE national_id = $1
            RETURNING
                national_id,
                full_name,
                main_phone_no,
                secondary_phone_no,
                email,
                address,
                role_id,
                salary
            "#,
            national_id,
        )
        .fetch_one(connection)
        .await
    }
}

#[async_trait]
impl Paginable<Employee> for Employee {
    async fn get_page(
        pages: &Pages<Employee, Employee>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<Employee>, sqlx::Error> {
        let page_items = sqlx::query_as!(
            Employee,
            r#"
                SELECT
                    national_id,
                    full_name,
                    main_phone_no,
                    secondary_phone_no,
                    email,
                    address,
                    role_id,
                    salary
                FROM staff
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
pub struct InsertEmployee {
    pub national_id: String,
    pub full_name: String,
    pub main_phone_no: String,
    pub secondary_phone_no: String,
    pub email: String,
    pub address: String,
    pub role_id: i32,
    pub salary: BigDecimal,
}

impl InsertEmployee {
    pub async fn insert(
        self,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Employee, sqlx::Error> {
        sqlx::query_as!(
            Employee,
            r#"
            INSERT INTO staff (
                national_id,
                full_name,
                main_phone_no,
                secondary_phone_no,
                email,
                address,
                role_id,
                salary
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8
            )
            RETURNING
                national_id,
                full_name,
                main_phone_no,
                secondary_phone_no,
                email,
                address,
                role_id,
                salary
            "#,
            self.national_id as _,
            self.full_name,
            self.main_phone_no as _,
            self.secondary_phone_no as _,
            self.email as _,
            self.address,
            self.role_id,
            self.salary,
        )
        .fetch_one(connection)
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateEmployee {
    pub national_id: Option<String>,
    pub full_name: Option<String>,
    pub main_phone_no: Option<String>,
    pub secondary_phone_no: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub role_id: Option<i32>,
    pub salary: Option<BigDecimal>,
}

impl UpdateEmployee {
    pub async fn update(
        self,
        target: Employee,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Employee, sqlx::Error> {
        let new_national_id = self.national_id.as_ref().unwrap_or(&target.national_id);
        let new_full_name = self.full_name.unwrap_or(target.full_name);
        let new_main_phone_no = self.main_phone_no.unwrap_or(target.main_phone_no);
        let new_secondary_phone_no = self.secondary_phone_no.unwrap_or(target.secondary_phone_no);
        let new_email = self.email.unwrap_or(target.email);
        let new_address = self.address.unwrap_or(target.address);
        let new_role_id = self.role_id.unwrap_or(target.role_id);
        let new_salary = self.salary.unwrap_or(target.salary);

        sqlx::query_as!(
            Employee,
            r#"
            UPDATE staff
            SET
                national_id = $1,
                full_name = $2,
                main_phone_no = $3,
                secondary_phone_no = $4,
                email = $5,
                address = $6,
                role_id = $7,
                salary = $8
            WHERE national_id = $9
            RETURNING
                national_id,
                full_name,
                main_phone_no,
                secondary_phone_no,
                email,
                address,
                role_id,
                salary
            "#,
            new_national_id as _,
            new_full_name,
            new_main_phone_no as _,
            new_secondary_phone_no as _,
            new_email as _,
            new_address,
            new_role_id,
            new_salary,
            target.national_id,
        )
        .fetch_one(connection)
        .await
    }
}
