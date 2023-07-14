use serde::Serialize;

use crate::services::service_error::ServiceError;

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct NonPaginatedResponseDto<T: Serialize> {
    pub data: T
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Pagination {
    pub total: u32,
    pub page: u32,
    pub pages: u32,
    pub per_page: u32
}

impl Pagination {
    pub fn new (total: u32, page: u32, per_page: u32) -> Pagination {
        Pagination { 
            total: total, 
            page: page, 
            pages: total / per_page + total % per_page, 
            per_page: per_page 
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct PaginatedResponseDto<T: Serialize> {
    pub data: T,
    pub pagination: Pagination
}

// #[derive(Serialize)]
// #[serde(rename_all="camelCase")]
// pub struct ErrorResponseDto {
//     pub error: ServiceError
// }