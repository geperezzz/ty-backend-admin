use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct PaginationParams {
    pub per_page: Option<i64>,
    pub page_no: Option<i64>
}