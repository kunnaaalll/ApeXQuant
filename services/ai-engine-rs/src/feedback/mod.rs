use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionFeedback {
    pub strategy_id: String,
    pub expected_price: Decimal,
    pub realized_price: Decimal,
    pub slippage: Decimal,
    pub latency_ms: u64,
}

pub struct FeedbackEngine;

impl FeedbackEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn process_execution_feedback(&self, feedback: &ExecutionFeedback) -> bool {
        // Deterministic analysis: flag if slippage is > 0.05%
        let slippage_threshold = feedback.expected_price * Decimal::new(5, 4); // 0.0005
        
        if feedback.slippage > slippage_threshold {
            // High slippage detected, we could trigger a regime re-evaluation or strategy pause
            return true;
        }
        
        false
    }
}
