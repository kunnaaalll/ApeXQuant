use thiserror::Error;

#[derive(Error, Debug)]
pub enum RiskError {
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}
