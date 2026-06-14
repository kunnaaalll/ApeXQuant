use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleOutRecommendation {
    pub should_scale_out: bool,
    pub target_reduction_pct: Decimal, // 0.25, 0.50, 0.75, etc.
    pub reason: String,
}

pub struct ScaleOutEngine;

impl ScaleOutEngine {
    pub fn evaluate(distance_to_target_pct: f32) -> ScaleOutRecommendation {
        if distance_to_target_pct <= 0.10 {
            ScaleOutRecommendation {
                should_scale_out: true,
                target_reduction_pct: Decimal::new(50, 2), // 50%
                reason: "Position is within 10% of target. Recommend securing 50% profits."
                    .to_string(),
            }
        } else {
            ScaleOutRecommendation {
                should_scale_out: false,
                target_reduction_pct: Decimal::ZERO,
                reason: "Position has room to run.".to_string(),
            }
        }
    }
}
