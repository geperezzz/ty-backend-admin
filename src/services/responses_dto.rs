use serde::Serialize;
use crate::models::pagination::Pagination;

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct NonPaginatedResponseDto<T: Serialize> {
    pub data: T
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct PaginatedResponseDto<T: Serialize> {
    pub data: T,
    pub pagination: Pagination
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct ErrorResponseDto<T: Serialize> {
    pub data: T
}