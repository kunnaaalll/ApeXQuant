//! Historical Replay Engine - Replay and validate market data
//!
//! Provides deterministic replay of historical market data for:
//! - Determinism validation
//! - Parity testing against TypeScript reference
//! - Performance benchmarking
//! - Stress testing

use crate::market_data::Candle;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

pub mod loader;
pub mod metrics;
pub mod runner;

pub use loader::{DatasetLoader, HistoricalDataset};
pub use metrics::{ReplayMetrics, ReplayStatistics};
pub use runner::{ReplayConfig, ReplayEngine, ReplayResult};

/// Errors that can occur during replay
#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("Dataset not found: {0}")]
    DatasetNotFound(String),
    #[error("Invalid dataset format: {0}")]
    InvalidFormat(String),
    #[error("Replay failed: {0}")]
    ReplayFailed(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ReplayError>;

/// A single market data point in a replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataPoint {
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub timeframe: String,
    pub candle: Candle,
    pub expected_signal: Option<ExpectedSignal>,
}

/// Expected signal from reference implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedSignal {
    pub direction: String,
    pub confidence: f64,
    pub confluence_score: f64,
    pub entry_price: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub patterns: Vec<String>,
    pub regime: String,
}

/// A test scenario from the golden dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenario {
    pub scenario_id: String,
    pub name: String,
    pub description: String,
    pub category: ScenarioCategory,
    pub symbol: String,
    pub timeframe: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub candles: Vec<Candle>,
    pub expected_signals: Vec<ExpectedSignal>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Category of test scenario
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScenarioCategory {
    /// Strong uptrend
    TrendingUp,
    /// Strong downtrend
    TrendingDown,
    /// Choppy/sideways market
    Ranging,
    /// High volatility expansion
    HighVolatility,
    /// Low volatility contraction
    LowVolatility,
    /// Breakout pattern
    Breakout,
    /// Liquidity sweep scenario
    LiquiditySweep,
    /// Strong BOS event
    StrongBos,
    /// Strong CHOCH event
    StrongChoch,
    /// Valid order block zone
    GoodOrderBlock,
    /// Valid fair value gap
    GoodFvg,
    /// Complex multi-pattern scenario
    Complex,
}

/// Dataset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub name: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub total_scenarios: usize,
    pub categories: Vec<ScenarioCategory>,
    pub symbols: Vec<String>,
    pub timeframes: Vec<String>,
    pub total_candles: usize,
    pub total_expected_signals: usize,
}

/// Golden dataset collection
pub struct GoldenDataset {
    pub metadata: DatasetMetadata,
    pub scenarios: Vec<TestScenario>,
    pub by_category: HashMap<ScenarioCategory, Vec<usize>>,
    pub by_symbol: HashMap<String, Vec<usize>>,
}

impl GoldenDataset {
    /// Create a new empty golden dataset
    pub fn new(name: String) -> Self {
        Self {
            metadata: DatasetMetadata {
                name,
                version: "1.0.0".to_string(),
                created_at: Utc::now(),
                total_scenarios: 0,
                categories: Vec::new(),
                symbols: Vec::new(),
                timeframes: Vec::new(),
                total_candles: 0,
                total_expected_signals: 0,
            },
            scenarios: Vec::new(),
            by_category: HashMap::new(),
            by_symbol: HashMap::new(),
        }
    }

    /// Add a scenario to the dataset
    pub fn add_scenario(&mut self, scenario: TestScenario) {
        let idx = self.scenarios.len();

        self.metadata.total_candles += scenario.candles.len();
        self.metadata.total_expected_signals += scenario.expected_signals.len();

        if !self.metadata.categories.contains(&scenario.category) {
            self.metadata.categories.push(scenario.category);
        }
        if !self.metadata.symbols.contains(&scenario.symbol) {
            self.metadata.symbols.push(scenario.symbol.clone());
        }
        if !self.metadata.timeframes.contains(&scenario.timeframe) {
            self.metadata.timeframes.push(scenario.timeframe.clone());
        }

        self.by_category
            .entry(scenario.category)
            .or_insert_with(Vec::new)
            .push(idx);

        self.by_symbol
            .entry(scenario.symbol.clone())
            .or_insert_with(Vec::new)
            .push(idx);

        self.scenarios.push(scenario);
        self.metadata.total_scenarios = self.scenarios.len();
    }

    /// Get scenarios by category
    pub fn get_by_category(&self, category: ScenarioCategory) -> Vec<&TestScenario> {
        self.by_category
            .get(&category)
            .map(|indices| indices.iter().map(|&i| &self.scenarios[i]).collect())
            .unwrap_or_default()
    }

    /// Get scenarios by symbol
    pub fn get_by_symbol(&self, symbol: &str) -> Vec<&TestScenario> {
        self.by_symbol
            .get(symbol)
            .map(|indices| indices.iter().map(|&i| &self.scenarios[i]).collect())
            .unwrap_or_default()
    }

    /// Shuffle scenarios deterministically (for test stability)
    pub fn shuffle_deterministic(&mut self, seed: u64) {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        // Simple deterministic shuffle based on seed
        let n = self.scenarios.len();
        for i in (1..n).rev() {
            let mut hasher = DefaultHasher::new();
            seed.hash(&mut hasher);
            i.hash(&mut hasher);
            let hash = hasher.finish();
            let j = (hash as usize) % (i + 1);
            self.scenarios.swap(i, j);
        }

        // Rebuild indices
        self.rebuild_indices();
    }

    fn rebuild_indices(&mut self) {
        self.by_category.clear();
        self.by_symbol.clear();

        for (idx, scenario) in self.scenarios.iter().enumerate() {
            self.by_category
                .entry(scenario.category)
                .or_insert_with(Vec::new)
                .push(idx);

            self.by_symbol
                .entry(scenario.symbol.clone())
                .or_insert_with(Vec::new)
                .push(idx);
        }
    }
}

/// Replay speed configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplaySpeed {
    /// Replay as fast as possible
    Maximum,
    /// Fixed steps per second
    FixedRate(u32),
    /// Real-time (1x speed)
    RealTime,
    /// Paused (step manually)
    Paused,
}

/// Verification level for replay
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationLevel {
    /// No verification, just run
    None,
    /// Basic direction verification only
    DirectionOnly,
    /// Direction + Confidence within tolerance
    DirectionAndConfidence,
    /// Full signal comparison
    Full,
}

/// Determinism test configuration
#[derive(Debug, Clone)]
pub struct DeterminismTest {
    pub iterations: u32,
    pub scenario: TestScenario,
    pub tolerance: f64,
}

/// Stress test configuration
#[derive(Debug, Clone)]
pub struct StressTestConfig {
    pub total_candles: usize,
    pub symbols: Vec<String>,
    pub timeframes: Vec<String>,
    pub concurrent_replays: usize,
    pub target_duration_seconds: u64,
}

/// Performance benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub warmup_iterations: u32,
    pub measurement_iterations: u32,
    pub scenarios: Vec<TestScenario>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_dataset_add_scenario() {
        let mut dataset = GoldenDataset::new("test".to_string());
        assert_eq!(dataset.metadata.total_scenarios, 0);

        let scenario = TestScenario {
            scenario_id: "test-1".to_string(),
            name: "Test".to_string(),
            description: "Test scenario".to_string(),
            category: ScenarioCategory::TrendingUp,
            symbol: "EURUSD".to_string(),
            timeframe: "M15".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            candles: vec![], // Empty for test
            expected_signals: vec![],
            metadata: HashMap::new(),
        };

        dataset.add_scenario(scenario.clone());
        assert_eq!(dataset.metadata.total_scenarios, 1);
        assert_eq!(dataset.get_by_category(ScenarioCategory::TrendingUp).len(), 1);
    }
}
