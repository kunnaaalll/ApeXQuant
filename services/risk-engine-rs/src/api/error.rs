use tonic::Status;

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    InvalidArgument,
    ValidationFailed,
    Unauthorized,
    Internal,
}

impl From<ApiError> for Status {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::NotFound => Status::not_found("Resource not found"),
            ApiError::InvalidArgument => Status::invalid_argument("Invalid argument"),
            ApiError::ValidationFailed => Status::invalid_argument("Validation failed"),
            ApiError::Unauthorized => Status::unauthenticated("Unauthorized"),
            ApiError::Internal => Status::internal("Internal error"),
        }
    }
}
