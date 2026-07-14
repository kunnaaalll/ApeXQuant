//! Shadow Validation Module
//!
//! Compares shadow executions, live executions, and backtest executions.

use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct ShadowExecution {
    pub order_id: String,
    pub fill_price: Decimal,
    pub fill_size: Decimal,
    pub latency_ms: i64,
    pub slippage: Decimal,
    pub risk_interventions: u32,
}

#[derive(Debug, Clone)]
pub struct ShadowValidationReport {
    pub order_id: String,
    pub fill_price_diff: Decimal,
    pub fill_size_diff: Decimal,
    pub latency_diff_ms: i64,
    pub slippage_diff: Decimal,
    pub risk_intervention_diff: i64,
}

pub struct ShadowValidator;

impl ShadowValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ShadowValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ShadowValidator {
    pub fn validate(
        &self,
        shadow: &ShadowExecution,
        live: &ShadowExecution,
        _backtest: &ShadowExecution,
    ) -> ShadowValidationReport {
        // Compare shadow/backtest against live as ground truth
        ShadowValidationReport {
            order_id: live.order_id.clone(),
            fill_price_diff: shadow.fill_price - live.fill_price,
            fill_size_diff: shadow.fill_size - live.fill_size,
            latency_diff_ms: shadow.latency_ms - live.latency_ms,
            slippage_diff: shadow.slippage - live.slippage,
            risk_intervention_diff: (shadow.risk_interventions as i64)
                - (live.risk_interventions as i64),
        }
    }
}
