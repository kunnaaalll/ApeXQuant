//! Portfolio Simulation Module
//!
//! Track equity curves, drawdowns, portfolio heat, exposure, and margin usage using Decimal.

use rust_decimal::Decimal;

pub struct PortfolioState {
    pub equity: Decimal,
    pub drawdown: Decimal,
    pub heat: Decimal,
    pub exposure: Decimal,
    pub margin_usage: Decimal,
}
