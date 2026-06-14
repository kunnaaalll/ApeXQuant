use serde::{Deserialize, Serialize};

/// Represents the overall health of a position on a 0-100 scale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScore {
    pub value: u8, // 0-100

    // Components (0-100 scale each)
    pub trend_alignment: u8,
    pub regime_quality: u8,
    pub momentum_strength: u8,
    pub drawdown_impact: u8,
    pub time_decay_impact: u8,
}

pub struct HealthScoreEngine;

impl HealthScoreEngine {
    /// Calculates a composite health score from multiple inputs.
    pub fn calculate(
        trend_alignment: u8,
        regime_quality: u8,
        momentum_strength: u8,
        drawdown_impact: u8,
        time_decay_impact: u8,
    ) -> HealthScore {
        // Weighted average (simplified placeholder for exact quantitative model)
        // Ensure weights sum to 1.0 implicitly through integer math where possible
        // e.g., trend 30%, regime 20%, momentum 20%, drawdown 20%, time decay 10%
        let value = ((trend_alignment as f32 * 0.30)
            + (regime_quality as f32 * 0.20)
            + (momentum_strength as f32 * 0.20)
            + (drawdown_impact as f32 * 0.20)
            + (time_decay_impact as f32 * 0.10))
            .clamp(0.0, 100.0) as u8;

        HealthScore {
            value,
            trend_alignment,
            regime_quality,
            momentum_strength,
            drawdown_impact,
            time_decay_impact,
        }
    }
}
