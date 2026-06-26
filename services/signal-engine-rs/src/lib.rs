//! APEX V3 Signal Engine
//!
//! Deterministic, institutional-grade signal generation for intraday swing trading.
//! Generates high-quality trade opportunities using market structure, multi-timeframe
//! analysis, and Smart Money Concepts (SMC).

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod api;
pub mod config;
pub mod error;
pub mod health;
pub mod market_data;
pub mod metrics;
pub mod signals;
pub mod event_bus;

pub use config::Config;
pub use error::{SignalEngineError, Result};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::market_data::CandleBuffer;
use crate::signals::{SignalGenerator, SignalResult};

/// Core signal engine handle
#[derive(Clone)]
pub struct SignalEngine {
    config: Arc<Config>,
    candle_buffers: Arc<RwLock<CandleBuffer>>,
    signal_generator: Arc<SignalGenerator>,
    event_bus: Option<Arc<event_bus::EventBusPublisher>>,
}

impl SignalEngine {
    /// Initialize the signal engine with the given configuration
    pub async fn new(config: Config, event_bus: Option<Arc<event_bus::EventBusPublisher>>) -> Result<Self> {
        let candle_buffers = Arc::new(RwLock::new(CandleBuffer::new(&config)));
        let signal_generator = Arc::new(SignalGenerator::new(&config));

        info!(
            "SignalEngine initialized with config: min_score={}, timeframes={:?}",
            config.min_confluence_score, config.timeframes
        );

        Ok(Self {
            config: Arc::new(config),
            candle_buffers,
            signal_generator,
            event_bus,
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

        Ok(vec![])
    }

    /// Get current health status
    pub async fn health(&self) -> health::HealthStatus {
        health::check_health(self).await
    }

    /// Get metrics snapshot
    pub fn metrics(&self) -> metrics::SignalMetrics {
        metrics::SignalMetrics::current()
    }
}
