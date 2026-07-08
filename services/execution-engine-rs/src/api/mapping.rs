use crate::api::errors::ApiError;
use rust_decimal::Decimal;
use std::str::FromStr;

pub fn parse_decimal(s: &str) -> Result<Decimal, ApiError> {
    Decimal::from_str(s)
        .map_err(|e| ApiError::Validation(format!("Invalid decimal '{}': {}", s, e)))
}

pub fn format_decimal(d: Decimal) -> String {
    d.to_string()
}
