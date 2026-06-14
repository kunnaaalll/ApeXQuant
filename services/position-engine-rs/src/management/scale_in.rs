use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::health::PositionQuality;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleInRecommendation {
    pub should_scale: bool,
    pub additional_size: Decimal,
    pub reason: String,
}

pub struct ScaleInEngine;

impl ScaleInEngine {
    pub fn evaluate(
        quality: &PositionQuality,
        current_risk_exposure: Decimal,
        max_risk_allowed: Decimal,
    ) -> ScaleInRecommendation {
        if *quality != PositionQuality::Excellent {
            return ScaleInRecommendation {
                should_scale: false,
                additional_size: Decimal::ZERO,
                reason: "Quality is not excellent; scaling in is unjustified.".to_string(),
            };
        }

        if current_risk_exposure >= max_risk_allowed {
            return ScaleInRecommendation {
                should_scale: false,
                additional_size: Decimal::ZERO,
                reason: "Maximum risk exposure already reached.".to_string(),
            };
        }

        // Placeholder for calculating how much to add
        let added_size = Decimal::new(10, 0); // e.g., 10 units

        ScaleInRecommendation {
            should_scale: true,
            additional_size: added_size,
            reason: "Trade quality is excellent and risk allows for scaling in.".to_string(),
        }
    }
}
