use crate::confidence::MarketConfidence;
use crate::liquidity::LiquidityGrade;
use crate::volatility::VolatilityGrade;
use crate::efficiency::EfficiencyGrade;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarketGrade {
    Elite,
    Strong,
    Normal,
    Weak,
    Poor,
}

pub struct MarketScoreMetrics {
    pub score: u32,
    pub grade: MarketGrade,
}

pub struct ScoringEngine;

impl ScoringEngine {
    pub fn evaluate(
        quality_score: u32,
        confidence_score: u32,
        liquidity_score: u32,
        vol_grade: VolatilityGrade,
        efficiency_grade: EfficiencyGrade
    ) -> Result<MarketScoreMetrics, &'static str> {
        
        let vol_penalty = match vol_grade {
            VolatilityGrade::Extreme => 20,
            VolatilityGrade::High => 10,
            _ => 0,
        };

        let eff_penalty = match efficiency_grade {
            EfficiencyGrade::Broken => 20,
            EfficiencyGrade::Noisy => 10,
            _ => 0,
        };

        let base_score = (quality_score + confidence_score + liquidity_score) / 3;
        
        let penalty = vol_penalty + eff_penalty;
        let score = base_score.saturating_sub(penalty);

        let clamped = score.clamp(0, 100);

        let grade = match clamped {
            s if s >= 90 => MarketGrade::Elite,
            s if s >= 70 => MarketGrade::Strong,
            s if s >= 40 => MarketGrade::Normal,
            s if s >= 20 => MarketGrade::Weak,
            _ => MarketGrade::Poor,
        };

        Ok(MarketScoreMetrics {
            score: clamped,
            grade,
        })
    }
}
