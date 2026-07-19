// src/analytics/snapshot.rs
use serde::{Deserialize, Serialize};

use super::efficiency::EfficiencyAssessment;
use super::expectancy::ExpectancyAssessment;
use super::performance::PerformanceAssessment;
use super::portfolio_metrics::PortfolioMetrics;
use super::regime::RegimePerformanceProfile;
use super::streak::StreakAnalytics;
use super::symbol::SymbolPerformanceProfile;
use super::timeframe::TimeframePerformanceProfile;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SnapshotResolution {
    Realtime,
    M1,
    M5,
    M15,
    H1,
    D1,
}

/// An immutable, versioned, timestamped snapshot of all analytics
/// to support deterministic replays and audits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyticsSnapshot {
    pub timestamp: i64,
    pub version: u64,
    pub resolution: SnapshotResolution,

    pub metrics: PortfolioMetrics,
    pub expectancy: ExpectancyAssessment,
    pub performance: PerformanceAssessment,
    pub efficiency: EfficiencyAssessment,
    pub regime: RegimePerformanceProfile,
    pub symbol: SymbolPerformanceProfile,
    pub timeframe: TimeframePerformanceProfile,
    pub streaks: StreakAnalytics,
}

impl AnalyticsSnapshot {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        timestamp: i64,
        version: u64,
        resolution: SnapshotResolution,
        metrics: PortfolioMetrics,
        expectancy: ExpectancyAssessment,
        performance: PerformanceAssessment,
        efficiency: EfficiencyAssessment,
        regime: RegimePerformanceProfile,
        symbol: SymbolPerformanceProfile,
        timeframe: TimeframePerformanceProfile,
        streaks: StreakAnalytics,
    ) -> Self {
        Self {
            timestamp,
            version,
            resolution,
            metrics,
            expectancy,
            performance,
            efficiency,
            regime,
            symbol,
            timeframe,
            streaks,
        }
    }
}
