//! Attribution Module
//!
//! Determine market, strategy, execution, and risk contributions.

use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct AttributionReport {
    pub strategy_edge: Decimal,
    pub market_regime_contribution: Decimal,
    pub execution_contribution: Decimal,
    pub risk_contribution: Decimal,
}
