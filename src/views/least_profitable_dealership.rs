use bigdecimal::BigDecimal;
use serde::Serialize;
use sqlx::{Executor, Postgres};
use time::Date;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeastProfitableDealership {
    pub rif: String,
    pub name: String,
    pub profit: BigDecimal,
}

impl LeastProfitableDealership {
    pub async fn select_all_in_range(
        from_date: Date,
        to_date: Date,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Vec<LeastProfitableDealership>, sqlx::Error> {
        sqlx::query_as!(
            LeastProfitableDealership,
            r#"
            WITH profits AS (
                SELECT
                    d.rif,
                    d.name,
                    SUM(i.amount_due) AS profit
                FROM
                    invoices AS i
                    INNER JOIN orders AS o ON i.order_id = o.id
                    INNER JOIN dealerships AS d ON o.dealership_rif = d.rif
                WHERE
                    i.issue_date BETWEEN $1 AND $2
                GROUP BY
                    d.rif,
                    d.name
            )
            SELECT
                rif,
                name,
                profit AS "profit!"
            FROM
                profits
            WHERE
                profit = (SELECT MIN(profit) FROM profits);
            "#,
            from_date,
            to_date
        )
        .fetch_all(connection)
        .await
    }
}
