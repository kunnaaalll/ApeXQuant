use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedQuality {
    Elite,
    Strong,
    Normal,
    Weak,
    Corrupted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityMetrics {
    pub stale_ticks: u64,
    pub duplicate_sequence_numbers: u64,
    pub gaps: u64,
    pub latency_ms: Decimal,
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl QualityMetrics {
    pub fn new() -> Self {
        Self {
            stale_ticks: 0,
            duplicate_sequence_numbers: 0,
            gaps: 0,
            latency_ms: Decimal::ZERO,
        }
    }

    pub fn evaluate(&self) -> FeedQuality {
        if self.duplicate_sequence_numbers > 50 || self.gaps > 20 {
            return FeedQuality::Corrupted;
        }
        if self.stale_ticks > 20 || self.latency_ms > Decimal::from(200) {
            return FeedQuality::Weak;
        }
        if self.stale_ticks > 5 || self.gaps > 5 || self.latency_ms > Decimal::from(50) {
            return FeedQuality::Normal;
        }
        if self.stale_ticks > 0 || self.gaps > 0 || self.latency_ms > Decimal::from(10) {
            return FeedQuality::Strong;
        }
        FeedQuality::Elite
    }
}

use crate::spread::SpreadGrade;
use crate::volatility::VolatilityGrade;
use crate::depth::DepthGrade;
use crate::liquidity::LiquidityGrade;
use crate::trend::TrendDirection;
use crate::noise::NoiseState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarketQualityGrade {
    Elite,
    Strong,
    Normal,
    Weak,
    Poor,
}

pub struct MarketQualityMetrics {
    pub score: u32,
    pub grade: MarketQualityGrade,
}

pub struct QualityEngine;

impl QualityEngine {
    pub fn calculate(
        spread: SpreadGrade, 
        volatility: VolatilityGrade, 
        depth: DepthGrade, 
        liquidity: LiquidityGrade, 
        _trend: TrendDirection, 
        noise: NoiseState
    ) -> Result<MarketQualityMetrics, &'static str> {
        let mut score = 0;

        score += match spread {
            SpreadGrade::Elite => 20,
            SpreadGrade::Strong => 15,
            SpreadGrade::Normal => 10,
            SpreadGrade::Weak => 5,
            SpreadGrade::Poor => 0,
        };

        score += match volatility {
            VolatilityGrade::VeryLow | VolatilityGrade::Low => 20,
            VolatilityGrade::Normal => 15,
            VolatilityGrade::High => 5,
            VolatilityGrade::Extreme => 0,
        };

        score += match depth {
            DepthGrade::Deep => 20,
            DepthGrade::Normal => 15,
            DepthGrade::Thin => 5,
            DepthGrade::Critical => 0,
        };

        score += match liquidity {
            LiquidityGrade::Institutional => 20,
            LiquidityGrade::Strong => 15,
            LiquidityGrade::Normal => 10,
            LiquidityGrade::Weak => 5,
            LiquidityGrade::Broken => 0,
        };

        score += match noise {
            NoiseState::Clean => 20,
            NoiseState::Moderate => 15,
            NoiseState::Noisy => 5,
            NoiseState::ExtremeNoise => 0,
        };

        let clamped = score.clamp(0, 100);

        let grade = match clamped {
            s if s >= 90 => MarketQualityGrade::Elite,
            s if s >= 70 => MarketQualityGrade::Strong,
            s if s >= 40 => MarketQualityGrade::Normal,
            s if s >= 20 => MarketQualityGrade::Weak,
            _ => MarketQualityGrade::Poor,
        };

        Ok(MarketQualityMetrics {
            score: clamped,
            grade,
        })
    }
}
