use crate::spread::SpreadGrade;
use crate::depth::DepthGrade;
use crate::volatility::VolatilityGrade;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LiquidityGrade {
    Institutional,
    Strong,
    Normal,
    Weak,
    Broken,
}

pub struct LiquidityMetrics {
    pub score: u32,
    pub grade: LiquidityGrade,
}

pub struct LiquidityEngine;

impl LiquidityEngine {
    pub fn calculate(spread: SpreadGrade, depth: DepthGrade, volatility: VolatilityGrade) -> Result<LiquidityMetrics, &'static str> {
        let spread_score = match spread {
            SpreadGrade::Elite => 40,
            SpreadGrade::Strong => 30,
            SpreadGrade::Normal => 20,
            SpreadGrade::Weak => 10,
            SpreadGrade::Poor => 0,
        };

        let depth_score = match depth {
            DepthGrade::Deep => 40,
            DepthGrade::Normal => 25,
            DepthGrade::Thin => 10,
            DepthGrade::Critical => 0,
        };

        let vol_score = match volatility {
            VolatilityGrade::VeryLow => 20,
            VolatilityGrade::Low => 20,
            VolatilityGrade::Normal => 15,
            VolatilityGrade::High => 5,
            VolatilityGrade::Extreme => 0,
        };

        let score = spread_score + depth_score + vol_score;
        let clamped_score = score.clamp(0, 100);

        let grade = match clamped_score {
            s if s >= 90 => LiquidityGrade::Institutional,
            s if s >= 70 => LiquidityGrade::Strong,
            s if s >= 40 => LiquidityGrade::Normal,
            s if s >= 20 => LiquidityGrade::Weak,
            _ => LiquidityGrade::Broken,
        };

        Ok(LiquidityMetrics {
            score: clamped_score,
            grade,
        })
    }
}
