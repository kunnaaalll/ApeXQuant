//! Parity Module
//!
//! Validators for Execution, Risk, Strategy, Market, and Portfolio models.

use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub enum ParityState {
    ExactMatch,
    CloseMatch,
    Warning,
    CriticalMismatch,
}

#[derive(Debug, Clone)]
pub struct ParityReport {
    pub is_valid: bool,
    pub parity_score: Decimal,
    pub drift_score: Decimal,
    pub confidence_score: Decimal,
    pub state: ParityState,
    pub max_deviation: Decimal,
    pub errors: Vec<String>,
}

pub trait ParityValidator {
    type Input;
    fn validate(&self, live: &Self::Input, simulated: &Self::Input) -> ParityReport;
}

pub struct ExecutionParityValidator {
    pub max_latency_variance_ms: i64,
    pub max_slippage_deviation: Decimal,
}

impl ExecutionParityValidator {
    pub fn new(max_latency_variance_ms: i64, max_slippage_deviation: Decimal) -> Self {
        Self {
            max_latency_variance_ms,
            max_slippage_deviation,
        }
    }
}

pub struct RiskParityValidator {
    pub max_exposure_deviation: Decimal,
}

impl RiskParityValidator {
    pub fn new(max_exposure_deviation: Decimal) -> Self {
        Self { max_exposure_deviation }
    }
}

pub struct StrategyParityValidator {
    pub max_signal_timing_deviation_ms: i64,
}

impl StrategyParityValidator {
    pub fn new(max_signal_timing_deviation_ms: i64) -> Self {
        Self { max_signal_timing_deviation_ms }
    }
}

pub struct PortfolioParityValidator {
    pub max_pnl_deviation: Decimal,
}

impl PortfolioParityValidator {
    pub fn new(max_pnl_deviation: Decimal) -> Self {
        Self { max_pnl_deviation }
    }
}

pub struct MarketParityValidator {
    pub max_price_deviation: Decimal,
}

impl MarketParityValidator {
    pub fn new(max_price_deviation: Decimal) -> Self {
        Self { max_price_deviation }
    }
}

impl ParityValidator for MarketParityValidator {
    type Input = rust_decimal::Decimal;

    fn validate(&self, live: &Self::Input, simulated: &Self::Input) -> ParityReport {
        let diff = (live - simulated).abs();
        let is_valid = diff <= self.max_price_deviation;
        ParityReport {
            is_valid,
            parity_score: rust_decimal_macros::dec!(100.0) - diff,
            drift_score: diff,
            confidence_score: rust_decimal_macros::dec!(0.99),
            state: if diff == rust_decimal_macros::dec!(0) { ParityState::ExactMatch } else { ParityState::CloseMatch },
            max_deviation: diff,
            errors: vec![],
        }
    }
}
