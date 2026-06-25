use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::MathematicalOps;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeUnderWaterAssessment {
    pub duration_seconds: u64,
    pub recovery_speed_per_day: Decimal,
    pub efficiency: Decimal,
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
    pub persistence: Decimal,
    pub ulcer_index: Decimal,
}

impl UlcerIndexAssessment {
    pub fn calculate(drawdowns_pct: &[Decimal]) -> Self {
        if drawdowns_pct.is_empty() {
            return Self {
                depth: Decimal::ZERO,
                duration: 0,
                persistence: Decimal::ZERO,
                ulcer_index: Decimal::ZERO,
            };
        }

        let mut sum_sq = Decimal::ZERO;
        let mut max_depth = Decimal::ZERO;
        let duration = drawdowns_pct.len() as u64;
        let mut nonzero_count = 0u64;

        for &dd in drawdowns_pct {
            if dd > Decimal::ZERO {
                nonzero_count += 1;
                sum_sq += dd * dd;
                if dd > max_depth {
                    max_depth = dd;
                }
            }
        }

        let duration_dec = Decimal::from_u64(duration).unwrap_or(Decimal::ONE);
        let mean_sq = sum_sq / duration_dec;
        
        let ulcer_index = mean_sq.sqrt().unwrap_or(Decimal::ZERO);
        
        let persistence = if duration > 0 {
            let nz_dec = Decimal::from_u64(nonzero_count).unwrap_or(Decimal::ZERO);
            nz_dec / duration_dec
        } else {
            Decimal::ZERO
        };

        Self {
            depth: max_depth,
            duration,
            persistence,
            ulcer_index,
        }
    }
}
