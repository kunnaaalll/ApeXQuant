use tonic::{Code, Status};

/// Errors originating from the strategy API layer.
/// This enum strictly defines the possible error states without leaking internal implementation details.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Invalid input provided: {0}")]
    InvalidInput(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Service is currently unavailable")]
    Unavailable,

    #[error("An internal server error occurred")]
    InternalError,
}

impl From<ApiError> for Status {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::InvalidInput(msg) => Status::new(Code::InvalidArgument, msg),
            ApiError::NotFound(msg) => Status::new(Code::NotFound, msg),
            ApiError::Unavailable => Status::new(Code::Unavailable, error.to_string()),
            // Internal errors do not leak their specific details or cause
            ApiError::InternalError => Status::new(Code::Internal, error.to_string()),
        }
    }
}
