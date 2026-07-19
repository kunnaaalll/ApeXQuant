use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QualityGrade {
    Elite,
    Excellent,
    Good,
    Average,
    Poor,
    Untradeable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketQualityMetrics {
    pub spread_quality: u8,
    pub liquidity_quality: u8,
    pub volatility_quality: u8,
    pub sequence_quality: u8,
    pub feed_health: u8,
    pub market_participation: u8,
    pub overall_score: u8,
    pub grade: QualityGrade,
}

#[derive(Debug, Clone)]
pub struct QualityEngine {}

impl Default for QualityEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub struct QualityEvaluationParams {
    pub spread: Decimal,
    pub average_spread: Decimal,
    pub liquidity_depth: Decimal,
    pub average_liquidity: Decimal,
    pub is_feed_healthy: bool,
    pub sequence_gaps: u32,
    pub current_volatility: Decimal,
    pub average_volatility: Decimal,
    pub current_participation: Decimal,
    pub average_participation: Decimal,
}

impl QualityEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate(
        &self,
        params: QualityEvaluationParams,
    ) -> Result<MarketQualityMetrics, &'static str> {
        // Ensure no zero division
        if params.average_spread.is_zero() {
            return Err("Average spread cannot be zero");
        }

        let spread_ratio = params.spread / params.average_spread;
        let spread_score = if spread_ratio < Decimal::ONE {
            100
        } else if spread_ratio < Decimal::from(2) {
            80
        } else if spread_ratio < Decimal::from(5) {
            40
        } else {
            10
        };

        let liq_ratio = if params.average_liquidity.is_zero() {
            Decimal::ZERO
        } else {
            params.liquidity_depth / params.average_liquidity
        };

        let liq_score = if liq_ratio > Decimal::from(2) {
            100
        } else if liq_ratio > Decimal::ONE {
            80
        } else if liq_ratio
            > rust_decimal::prelude::FromStr::from_str("0.5").unwrap_or(Decimal::ZERO)
        {
            50
        } else {
            20
        };

        let seq_score = if params.sequence_gaps == 0 {
            100
        } else if params.sequence_gaps < 3 {
            70
        } else {
            10
        };

        let feed_score = if params.is_feed_healthy { 100 } else { 0 };

        let vol_ratio = if params.average_volatility.is_zero() {
            Decimal::ONE
        } else {
            params.current_volatility / params.average_volatility
        };

        let vol_score = if vol_ratio > Decimal::from(3) {
            20 // Flash crash or extreme spike
        } else if vol_ratio > Decimal::from(2) {
            50
        } else if vol_ratio
            > rust_decimal::prelude::FromStr::from_str("0.5").unwrap_or(Decimal::ZERO)
        {
            100 // Healthy normal volatility
        } else {
            40 // Dead market
        };

        let part_ratio = if params.average_participation.is_zero() {
            Decimal::ONE
        } else {
            params.current_participation / params.average_participation
        };

        let part_score = if part_ratio > Decimal::from(2) {
            100
        } else if part_ratio > Decimal::ONE {
            80
        } else if part_ratio
            > rust_decimal::prelude::FromStr::from_str("0.5").unwrap_or(Decimal::ZERO)
        {
            50
        } else {
            20
        };

        let overall = ((spread_score as u16
            + liq_score as u16
            + seq_score as u16
            + feed_score as u16
            + vol_score as u16
            + part_score as u16)
            / 6) as u8;

        let grade = match overall {
            s if s >= 90 => QualityGrade::Elite,
            s if s >= 80 => QualityGrade::Excellent,
            s if s >= 60 => QualityGrade::Good,
            s if s >= 40 => QualityGrade::Average,
            s if s >= 20 => QualityGrade::Poor,
            _ => QualityGrade::Untradeable,
        };

        Ok(MarketQualityMetrics {
            spread_quality: spread_score,
            liquidity_quality: liq_score,
            volatility_quality: vol_score,
            sequence_quality: seq_score,
            feed_health: feed_score,
            market_participation: part_score,
            overall_score: overall,
            grade,
        })
    }
}
