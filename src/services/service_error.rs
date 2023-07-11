use actix_web::{
    ResponseError,
    http::StatusCode
};

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error("{0}")]
    ValidationError(String),
    #[error("")]
    UnexpectedError(#[from] anyhow::Error)
}

impl ResponseError for ServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ServiceError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}