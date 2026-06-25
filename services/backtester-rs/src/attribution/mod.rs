//! Attribution Module
//!
//! Determine market, strategy, execution, and risk contributions.

use rust_decimal::Decimal;

pub struct AttributionReport {
    pub market_contribution: Decimal,
    pub strategy_contribution: Decimal,
    pub execution_contribution: Decimal,
    pub risk_contribution: Decimal,
}
