//! Performance Module
//!
//! Metrics calculation strictly using Decimal arithmetic.

use rust_decimal::Decimal;

pub struct PerformanceMetrics {
    pub win_rate: Decimal,
    pub expectancy: Decimal,
    pub profit_factor: Decimal,
    pub average_rr: Decimal,
    pub drawdown: Decimal,
    pub recovery_factor: Decimal,
    pub sharpe_equivalent: Decimal,
}
