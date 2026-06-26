//! Production Validator Module
//!
//! Validates pre-deployment expectations against limits.

use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct ProductionExpectations {
    pub expected_slippage: Decimal,
    pub expected_latency_ms: i64,
    pub expected_drawdown: Decimal,
    pub expected_profit_factor: Decimal,
    pub expected_volatility: Decimal,
}

#[derive(Debug, Clone)]
pub struct ProductionValidationReport {
    pub is_ready: bool,
    pub production_readiness_score: Decimal,
    pub issues: Vec<String>,
}

pub struct ProductionValidator {
    pub limit_slippage: Decimal,
    pub limit_latency_ms: i64,
    pub limit_drawdown: Decimal,
    pub min_profit_factor: Decimal,
    pub limit_volatility: Decimal,
}

impl ProductionValidator {
    pub fn new(
        limit_slippage: Decimal,
        limit_latency_ms: i64,
        limit_drawdown: Decimal,
        min_profit_factor: Decimal,
        limit_volatility: Decimal,
    ) -> Self {
        Self {
            limit_slippage,
            limit_latency_ms,
            limit_drawdown,
            min_profit_factor,
            limit_volatility,
        }
    }

    pub fn validate(&self, expectations: &ProductionExpectations) -> ProductionValidationReport {
        let mut issues = Vec::new();
        let mut score = rust_decimal_macros::dec!(100.0);
        let deduction = rust_decimal_macros::dec!(20.0);

        if expectations.expected_slippage > self.limit_slippage {
            issues.push("Expected slippage exceeds limits.".to_string());
            score -= deduction;
        }

        if expectations.expected_latency_ms > self.limit_latency_ms {
            issues.push("Expected latency exceeds limits.".to_string());
            score -= deduction;
        }

        if expectations.expected_drawdown > self.limit_drawdown {
            issues.push("Expected drawdown exceeds limits.".to_string());
            score -= deduction;
        }

        if expectations.expected_profit_factor < self.min_profit_factor {
            issues.push("Expected profit factor is below minimum.".to_string());
            score -= deduction;
        }

        if expectations.expected_volatility > self.limit_volatility {
            issues.push("Expected volatility exceeds limits.".to_string());
            score -= deduction;
        }

        let is_ready = issues.is_empty();

        ProductionValidationReport {
            is_ready,
            production_readiness_score: score,
            issues,
        }
    }
}
