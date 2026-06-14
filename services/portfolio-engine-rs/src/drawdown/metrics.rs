use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeUnderWaterAssessment {
    pub duration_seconds: u64,
    pub recovery_speed_per_day: Decimal,
    pub efficiency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryFactor {
    pub net_profit: Decimal,
    pub max_drawdown: Decimal,
    pub recovery_speed: Decimal,
    pub ratio: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UlcerIndexAssessment {
    pub depth: Decimal,
    pub duration: u64,
    pub persistence: f64,
    pub ulcer_index: f64,
}

impl UlcerIndexAssessment {
    pub fn calculate(drawdowns_pct: &[f64]) -> Self {
        if drawdowns_pct.is_empty() {
            return Self {
                depth: Decimal::ZERO,
                duration: 0,
                persistence: 0.0,
                ulcer_index: 0.0,
            };
        }

        let mut sum_sq = 0.0;
        let mut max_depth = 0.0;
        let duration = drawdowns_pct.len() as u64;
        let mut nonzero_count = 0;

        for &dd in drawdowns_pct {
            if dd > 0.0 {
                nonzero_count += 1;
                sum_sq += dd * dd;
                if dd > max_depth {
                    max_depth = dd;
                }
            }
        }

        let mean_sq = sum_sq / (duration as f64);
        let ulcer_index = mean_sq.sqrt();
        let persistence = if duration > 0 {
            (nonzero_count as f64) / (duration as f64)
        } else {
            0.0
        };

        Self {
            depth: Decimal::from_f64_retain(max_depth).unwrap_or(Decimal::ZERO),
            duration,
            persistence,
            ulcer_index,
        }
    }
}
