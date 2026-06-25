use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceMetrics {
    pub sample_size_confidence: Decimal,
    pub statistical_confidence: Decimal,
    pub regime_confidence: Decimal,
    pub execution_confidence: Decimal,
}

pub struct ConfidenceEngine;

impl Default for ConfidenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfidenceEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compute_confidence(&self, metrics: &ConfidenceMetrics) -> u8 {
        let mut sum = Decimal::ZERO;
        sum += metrics.sample_size_confidence;
        sum += metrics.statistical_confidence;
        sum += metrics.regime_confidence;
        sum += metrics.execution_confidence;
        
        let avg = sum / Decimal::from(4);
        let score = (avg * Decimal::from(100)).round();
        
        let score_u32: u32 = score.try_into().unwrap_or(0);
        
        score_u32.min(100) as u8
    }
}
