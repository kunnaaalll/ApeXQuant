use tonic::Status;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Execution error: {0}")]
    Execution(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Broker error: {0}")]
    Broker(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    #[error("Unavailable: {0}")]
    Unavailable(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<ApiError> for Status {
    fn from(err: ApiError) -> Self {
        match err {
            ApiError::Validation(msg) => Status::invalid_argument(msg),
            ApiError::Execution(msg) => Status::internal(format!("Execution failed: {}", msg)),
            ApiError::Storage(msg) => Status::internal(format!("Storage error: {}", msg)),
            ApiError::Broker(msg) => Status::unavailable(format!("Broker error: {}", msg)),
            ApiError::NotFound(msg) => Status::not_found(msg),
            ApiError::PermissionDenied(msg) => Status::permission_denied(msg),
            ApiError::AlreadyExists(msg) => Status::already_exists(msg),
            ApiError::Unavailable(msg) => Status::unavailable(msg),
            ApiError::Internal(msg) => Status::internal(msg),
        }
    }
}
