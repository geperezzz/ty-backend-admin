use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Pagination {
    pub total: u32,
    pub page: u32,
    pub pages: u32,
    pub per_page: u32
}

