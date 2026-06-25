use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayMetrics {
    pub edge_decay: Decimal,
    pub confidence_decay: Decimal,
    pub regime_decay: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayOutput {
    pub decay_score: Decimal,
    pub urgency_score: Decimal,
}

pub struct DecayTracker;

impl Default for DecayTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl DecayTracker {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compute_decay(&self, metrics: &DecayMetrics) -> DecayOutput {
        // Average decay logic for illustration
        let mut sum = Decimal::ZERO;
        sum += metrics.edge_decay;
        sum += metrics.confidence_decay;
        sum += metrics.regime_decay;
        let avg_decay = sum / Decimal::from(3);

        DecayOutput {
            decay_score: avg_decay,
            urgency_score: avg_decay * Decimal::new(15, 1), // 1.5 multiplier for urgency
        }
    }
}
