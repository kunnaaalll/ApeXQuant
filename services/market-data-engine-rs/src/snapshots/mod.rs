// Snapshots

use crate::health::FeedHealthGrade;
use crate::session::TradingSession;
use crate::spread::SpreadGrade;
use crate::volatility::VolatilityGrade;
use crate::state_machine::ConnectionState;
use crate::failover::FailoverState;
use crate::latency::LatencyGrade;
use std::collections::VecDeque;

use crate::quality::QualityGrade;
use crate::liquidity::LiquidityGrade;
use crate::trend::TrendState;
use crate::momentum::MomentumGrade;
use crate::efficiency::EfficiencyGrade;
use crate::noise::NoiseState;
use crate::confidence::MarketConfidence;
use crate::market_state::MarketState;
use crate::regime::MarketRegime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketDataSnapshot {
    pub symbol: String,
    pub session_state: TradingSession,
    pub spread_state: SpreadGrade,
    pub volatility_state: VolatilityGrade,
    pub feed_health: FeedHealthGrade,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketSnapshot {
    pub symbol: String,
    pub state: MarketState,
    pub quality: QualityGrade,
    pub quality_score: u32,
    pub liquidity: LiquidityGrade,
    pub spread: SpreadGrade,
    pub volatility: VolatilityGrade,
    pub regime: MarketRegime,
    pub regime_confidence: u8,
    pub trend_state: TrendState,
    pub momentum_grade: MomentumGrade,
    pub efficiency: EfficiencyGrade,
    pub noise: NoiseState,
    pub confidence: MarketConfidence,
    pub timestamp_ns: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedSnapshot {
    pub id: String,
    pub connection_state: ConnectionState,
    pub health: FeedHealthGrade,
    pub latency: LatencyGrade,
    pub failover_status: FailoverState,
    pub registry_active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamingSnapshot {
    pub symbol: String,
    pub buffered_ticks: VecDeque<crate::tick::Tick>,
    pub sequence_state: crate::sequence::SequenceState,
    pub total_ticks: u64,
    pub duplicate_ticks: u64,
    pub gaps_detected: u64,
    pub candles_created: u64,
    pub replays_executed: u64,
    pub throughput_grade: crate::throughput::ThroughputGrade,
}

use crate::intelligence::MarketIntelligenceProfile;
use crate::regime::RegimeMetrics;
use crate::volatility::VolatilityMetrics;
use crate::trend::TrendMetrics;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VolatilitySnapshot {
    pub metrics: VolatilityMetrics,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrendSnapshot {
    pub metrics: TrendMetrics,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegimeSnapshot {
    pub metrics: RegimeMetrics,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketIntelligenceSnapshot {
    pub profile: MarketIntelligenceProfile,
    pub timestamp: i64,
}

// These snapshots support exact replay reconstruction by serializing/deserializing deterministically.
