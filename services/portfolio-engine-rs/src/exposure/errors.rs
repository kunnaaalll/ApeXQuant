use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ExposureError {
    #[error("Invariant violation: Total weight ({total_weight}) exceeds 100%")]
    WeightExceedsMax { total_weight: Decimal },
    #[error("Invariant violation: Gross exposure ({gross}) < Net exposure ({net})")]
    GrossLessThanNet { gross: Decimal, net: Decimal },
    #[error("Invariant violation: Position count cannot be negative")]
    NegativePositionCount,
    #[error("Invariant violation: Negative gross exposure is impossible")]
    NegativeGrossExposure,
    #[error("Synthetic exposure mismatch: Base and Quote amounts do not balance")]
    SyntheticImbalance,
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
    #[error("System Error: {0}")]
    SystemError(String),
}
