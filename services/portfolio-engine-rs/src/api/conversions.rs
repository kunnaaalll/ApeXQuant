use rust_decimal::Decimal;
use std::str::FromStr;
use tonic::Status;

pub fn decimal_to_string(decimal: Decimal) -> String {
    decimal.to_string()
}

pub fn string_to_decimal(s: &str) -> Result<Decimal, Status> {
    Decimal::from_str(s)
        .map_err(|e| Status::invalid_argument(format!("Invalid decimal value: {}", e)))
}

pub fn opt_string_to_decimal(s: Option<&String>) -> Result<Option<Decimal>, Status> {
    match s {
        Some(val) if !val.is_empty() => Ok(Some(string_to_decimal(val)?)),
        _ => Ok(None),
    }
}
