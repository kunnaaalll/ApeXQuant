//! Benchmark Module
//!
//! Compares strategy performance against standard benchmarks.

use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub enum BenchmarkType {
    BuyAndHold,
    EqualWeightPortfolio,
    RiskParityPortfolio,
    StrategyBasket,
}

#[derive(Debug, Clone)]
pub struct BenchmarkReport {
    pub benchmark_type: BenchmarkType,
    pub alpha_score: Decimal,
    pub relative_performance: Decimal,
    pub risk_adjusted_outperformance: Decimal,
}

pub struct BenchmarkEngine;

impl BenchmarkEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BenchmarkEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl BenchmarkEngine {
    pub fn generate_report(
        &self,
        benchmark_type: BenchmarkType,
        strategy_return: Decimal,
        benchmark_return: Decimal,
        strategy_risk: Decimal,
        benchmark_risk: Decimal,
    ) -> BenchmarkReport {
        let relative_performance = strategy_return - benchmark_return;

        // Simplified Risk Adjusted Return (e.g. Sharpe-like comparison)
        // Ensure no division by zero or panic if benchmark_risk is zero
        let risk_adjusted_outperformance = if benchmark_risk.is_zero() || strategy_risk.is_zero() {
            Decimal::ZERO
        } else {
            (strategy_return / strategy_risk) - (benchmark_return / benchmark_risk)
        };

        // Alpha could be more complex, simplistic for now
        let alpha_score = relative_performance;

        BenchmarkReport {
            benchmark_type,
            alpha_score,
            relative_performance,
            risk_adjusted_outperformance,
        }
    }
}
