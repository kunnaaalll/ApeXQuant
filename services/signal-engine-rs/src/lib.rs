//! APEX V3 Signal Engine
//!
//! Deterministic, institutional-grade signal generation for intraday swing trading.
//! Generates high-quality trade opportunities using market structure, multi-timeframe
//! analysis, and Smart Money Concepts (SMC).

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod api;
pub mod config;
pub mod confidence;
pub mod confluence;
pub mod error;
pub mod evidence;
pub mod filters;
pub mod health;
pub mod market_data;
pub mod metrics;
pub mod mtf;
pub mod parity;
pub mod regime;
pub mod replay;
pub mod scoring;
pub mod signals;
pub mod smc;
pub mod storage;
pub mod structure;

pub use config::Config;
pub use error::{SignalEngineError, Result};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::confidence::ConfidenceCalculator;
use crate::confluence::engine::ConfluenceEngine;
use crate::market_data::CandleBuffer;
use crate::mtf::MTFAnalyzer;
use crate::regime::RegimeDetector;
use crate::signals::{SignalGenerator, SignalResult};
use crate::smc::SMCEngine;
use crate::structure::StructureAnalyzer;

/// Core signal engine handle
#[derive(Clone)]
pub struct SignalEngine {
    config: Arc<Config>,
    candle_buffers: Arc<RwLock<CandleBuffer>>,
    structure_analyzer: Arc<StructureAnalyzer>,
    mtf_analyzer: Arc<MTFAnalyzer>,
    regime_detector: Arc<RegimeDetector>,
    signal_generator: Arc<SignalGenerator>,
    smc_engine: Arc<SMCEngine>,
    confluence_engine: Arc<ConfluenceEngine>,
    confidence_calculator: Arc<ConfidenceCalculator>,
}

impl SignalEngine {
    /// Initialize the signal engine with the given configuration
    pub async fn new(config: Config) -> Result<Self> {
        let candle_buffers = Arc::new(RwLock::new(CandleBuffer::new(&config)));
        let structure_analyzer = Arc::new(StructureAnalyzer::new(&config));
        let mtf_analyzer = Arc::new(MTFAnalyzer::new(&config));
        let regime_detector = Arc::new(RegimeDetector::new(&config));
        let signal_generator = Arc::new(SignalGenerator::new(&config));
        let smc_engine = Arc::new(SMCEngine::new());
        let confluence_engine = Arc::new(ConfluenceEngine::new(config.min_confluence_score));
        let confidence_calculator = Arc::new(ConfidenceCalculator::new());

        info!(
            "SignalEngine initialized with config: min_score={}, timeframes={:?}",
            config.min_confluence_score, config.timeframes
        );

        Ok(Self {
            config: Arc::new(config),
            candle_buffers,
            structure_analyzer,
            mtf_analyzer,
            regime_detector,
            signal_generator,
            smc_engine,
            confluence_engine,
            confidence_calculator,
        })
    }

    /// Process incoming market data and generate signals
    pub async fn process_market_data(
        &self,
        symbol: &str,
        timeframe: &str,
        candles: Vec<market_data::Candle>,
    ) -> Result<Vec<SignalResult>> {
        debug!(
            "Processing {} candles for {} {}",
            candles.len(),
            symbol,
            timeframe
        );

        // Update candle buffer
        {
            let mut buffers = self.candle_buffers.write().await;
            buffers.add_candles(symbol, timeframe, candles)?;
        }

        // Get current market context
        let context = self.build_context(symbol).await?;

        // Generate signals
        let signals = self
            .signal_generator
            .generate(&context)
            .await?;

        info!(
            "Generated {} signals for {} (context: {:?})",
            signals.len(),
            symbol,
            context.regime.regime_type
        );

        Ok(signals)
    }

    /// Build complete market context from all timeframes
    async fn build_context(&self, symbol: &str) -> Result<signals::MarketContext> {
        let buffers = self.candle_buffers.read().await;
        let candles = buffers.get_all_timeframes(symbol)?;

        let regime = self.regime_detector.detect(&candles)?;
        let structure = self.structure_analyzer.analyze(&candles)?;
        let mtf_alignment = self.mtf_analyzer.analyze(&candles, &structure)?;

        Ok(signals::MarketContext {
            symbol: symbol.to_string(),
            candles,
            regime,
            structure,
            mtf_alignment,
        })
    }

    /// Get current health status
    pub async fn health(&self) -> health::HealthStatus {
        health::check_health(self).await
    }

    /// Get metrics snapshot
    pub fn metrics(&self) -> metrics::SignalMetrics {
        metrics::SignalMetrics::current()
    }

    /// Get SMC analysis for a symbol
    pub async fn analyze_smc(&self, symbol: &str) -> Result<HashMap<String, smc::SMCAnalysis>> {
        let buffers = self.candle_buffers.read().await;
        let candles = buffers.get_all_timeframes(symbol)?;

        // Collect swings from all timeframes
        let mut swings = HashMap::new();
        for tf in self.config.timeframes.clone() {
            if let Ok(tf_candles) = buffers.get_candles(symbol, &tf) {
                let tf_swings = structure::swings::detect_swings(&tf_candles, self.config.swing_pivot_bars);
                let mut all_swings = tf_swings.highs;
                all_swings.extend(tf_swings.lows);
                swings.insert(tf, all_swings);
            }
        }

        Ok(self.smc_engine.analyze(&candles, &swings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_initialization() {
        let config = Config::default();
        let engine = SignalEngine::new(config).await;
        assert!(engine.is_ok());
    }
}
