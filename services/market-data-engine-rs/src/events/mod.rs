// Event Sourcing

use crate::candle::OHLCV;
use crate::health::FeedHealthGrade;
use crate::latency::LatencyGrade;
use crate::regime::MarketRegime;
use crate::session::TradingSession;
use crate::spread::SpreadGrade;
use crate::tick::Tick;
use crate::volatility::VolatilityGrade;
use crate::trend::TrendDirection;
use crate::liquidity::LiquidityGrade;
use crate::quality::MarketQualityGrade;
use crate::anomaly::AnomalySeverity;
use crate::market_state::MarketState;
use crate::confidence::MarketConfidence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarketDataEvent {
    TickReceived(Tick),
    CandleClosed(OHLCV),
    SessionChanged(TradingSession),
    RegimeDetected(MarketRegime),
    SpreadChanged(SpreadGrade),
    VolatilityChanged(VolatilityGrade),
    HealthChanged(FeedHealthGrade),
    MarketQualityUpdated(MarketQualityGrade),
    LiquidityChanged(LiquidityGrade),
    TrendUpdated(TrendDirection),
    AnomalyDetected(AnomalySeverity),
    MarketStateChanged(MarketState),
    ConfidenceChanged(MarketConfidence),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedEvent {
    Connected(String),
    Disconnected(String),
    SynchronizationStarted(String),
    SynchronizationCompleted(String),
    HealthChanged { symbol: String, grade: FeedHealthGrade },
    LatencyChanged { symbol: String, grade: LatencyGrade },
    FailoverTriggered { from_symbol: String, to_symbol: String },
    FeedRecovered(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamingEvent {
    TickArrived(Tick),
    TickBuffered(Tick),
    GapDetected(crate::gaps::GapEvent),
    ReplayStarted,
    ReplayCompleted,
    SequenceBroken(crate::sequence::SequenceState),
    CandleCreated(OHLCV),
    ThroughputChanged(crate::throughput::ThroughputGrade),
}
