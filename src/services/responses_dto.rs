use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NonPaginatedResponseDto<T: Serialize> {
    pub data: T,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub total: i64,
    pub page: i64,
    pub pages: i64,
    pub per_page: i64,
}

impl Pagination {
    pub fn new(total: i64, page: i64, per_page: i64) -> Pagination {
        Pagination {
            total: total,
            page: page,
            pages: total / per_page + total % per_page,
            per_page: per_page,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponseDto<T: Serialize> {
    pub data: T,
    pub pagination: Pagination,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponseDto {
    pub error: String,
}
