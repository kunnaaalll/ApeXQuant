use tonic::Status;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Storage failure: {0}")]
    Storage(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal failure: {0}")]
    Internal(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Dependency unavailable: {0}")]
    Unavailable(String),
}

impl From<ApiError> for Status {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::Validation(msg) => Status::invalid_argument(msg),
            ApiError::Storage(msg) => Status::internal(format!("Storage error: {}", msg)),
            ApiError::NotFound(msg) => Status::not_found(msg),
            ApiError::Internal(msg) => Status::internal(msg),
            ApiError::Timeout(msg) => Status::deadline_exceeded(msg),
            ApiError::Unavailable(msg) => Status::unavailable(msg),
        }
    }
}
