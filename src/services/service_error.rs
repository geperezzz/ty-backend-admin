use actix_web::{http::StatusCode, ResponseError};

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error("{0}")]
    DomainValidationError(String),
    #[error("There is not any {0} with the given id")]
    ResourceNotFound(String),
    #[error("{0}")]
    InvalidUpdateError(String),
    #[error("{0}")]
    InvalidCreateError(String),
    #[error("")]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for ServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::DomainValidationError(_) => StatusCode::BAD_REQUEST,
            ServiceError::ResourceNotFound(_) => StatusCode::NOT_FOUND,
            ServiceError::InvalidUpdateError(_) => StatusCode::BAD_REQUEST,
            ServiceError::InvalidCreateError(_) => StatusCode::BAD_REQUEST,
            ServiceError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
