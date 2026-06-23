#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarketConfidence {
    VeryHigh,
    High,
    Medium,
    Low,
    VeryLow,
}

pub struct ConfidenceMetrics {
    pub score: u32,
    pub grade: MarketConfidence,
}

pub struct ConfidenceEngine;

impl ConfidenceEngine {
    pub fn evaluate(regime_confidence: u32, noise_state: crate::noise::NoiseState, anomaly_count: u32) -> Result<ConfidenceMetrics, &'static str> {
        use crate::noise::NoiseState;
        
        let noise_penalty = match noise_state {
            NoiseState::Clean => 0,
            NoiseState::Moderate => 10,
            NoiseState::Noisy => 25,
            NoiseState::ExtremeNoise => 50,
        };

        let anomaly_penalty = anomaly_count * 10;
        
        let penalty = noise_penalty + anomaly_penalty;
        
        let score = regime_confidence.saturating_sub(penalty);
        
        let clamped = score.clamp(0, 100);

        let grade = match clamped {
            s if s >= 80 => MarketConfidence::VeryHigh,
            s if s >= 60 => MarketConfidence::High,
            s if s >= 40 => MarketConfidence::Medium,
            s if s >= 20 => MarketConfidence::Low,
            _ => MarketConfidence::VeryLow,
        };

        Ok(ConfidenceMetrics {
            score: clamped,
            grade,
        })
    }
}
