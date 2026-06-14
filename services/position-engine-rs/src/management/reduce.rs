use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::health::PositionQuality;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReduceRecommendation {
    pub should_reduce: bool,
    pub reduction_pct: Decimal,
    pub reason: String,
}

pub struct ReduceEngine;

impl ReduceEngine {
    pub fn evaluate(quality: &PositionQuality, volatility_increase: bool) -> ReduceRecommendation {
        if *quality == PositionQuality::Weak || volatility_increase {
            ReduceRecommendation {
                should_reduce: true,
                reduction_pct: Decimal::new(50, 2), // 50% reduction
                reason: "Quality is weak or volatility spiked. Reducing exposure to mitigate risk."
                    .to_string(),
            }
        } else {
            ReduceRecommendation {
                should_reduce: false,
                reduction_pct: Decimal::ZERO,
                reason: "No reduction necessary.".to_string(),
            }
        }
    }
}
