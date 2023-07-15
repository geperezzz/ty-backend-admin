use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};

use super::responses_dto::ErrorResponseDto;

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error("{0}")]
    DomainValidationError(String),
    #[error("There is not any {0} with the given id")]
    ResourceNotFound(String),
    #[error("{0}")]
    MissingQueryParamError(String),
    #[error("{0}")]
    InvalidQueryParamValueError(String),
    #[error("{0}")]
    InvalidUpdateError(String),
    #[error("{0}")]
    InvalidCreateError(String),
    #[error("")]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .content_type(ContentType::json())
            .json(ErrorResponseDto {
                error: format!("{}", self),
            })
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::DomainValidationError(_) => StatusCode::BAD_REQUEST,
            ServiceError::ResourceNotFound(_) => StatusCode::NOT_FOUND,
            ServiceError::MissingQueryParamError(_) => StatusCode::BAD_REQUEST,
            ServiceError::InvalidQueryParamValueError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ServiceError::InvalidUpdateError(_) => StatusCode::BAD_REQUEST,
            ServiceError::InvalidCreateError(_) => StatusCode::BAD_REQUEST,
            ServiceError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
