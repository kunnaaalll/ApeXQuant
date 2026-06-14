use super::health_score::HealthScore;
use serde::{Deserialize, Serialize};

/// Qualitative classification of the position's health and potential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionQuality {
    Excellent,
    Good,
    Neutral,
    Weak,
    Critical,
}

pub struct QualityEngine;

impl QualityEngine {
    /// Determines qualitative classification based on the numeric health score.
    pub fn classify(score: &HealthScore) -> (PositionQuality, String) {
        match score.value {
            80..=100 => (
                PositionQuality::Excellent,
                "Strong momentum and regime alignment. Minimal drawdown.".to_string(),
            ),
            60..=79 => (
                PositionQuality::Good,
                "Acceptable performance, minor degradations in momentum or time decay.".to_string(),
            ),
            40..=59 => (
                PositionQuality::Neutral,
                "Stalling momentum or increasing drawdown. Watch closely.".to_string(),
            ),
            20..=39 => (
                PositionQuality::Weak,
                "Significant deterioration. Consider reducing exposure.".to_string(),
            ),
            0..=19 => (
                PositionQuality::Critical,
                "Thesis invalidated or extreme risk. Recommend immediate exit.".to_string(),
            ),
            _ => (
                PositionQuality::Neutral, // Fallback
                "Unable to classify accurately.".to_string(),
            ),
        }
    }
}
