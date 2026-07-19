#![allow(warnings, clippy::all, deprecated)]
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
pub mod event_bus;
pub mod health;
pub mod market_data;
pub mod metrics;
pub mod signals;
pub mod storage;

pub mod confidence;
pub mod confluence;
pub mod mtf;
pub mod parity;
pub mod regime;
pub mod smc;
pub mod structure;

pub use config::Config;
pub use error::{Result, SignalEngineError};

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::market_data::CandleBuffer;
use crate::signals::{SignalGenerator, SignalResult};
use crate::storage::SignalRepository;

/// Core signal engine handle
#[derive(Clone)]
pub struct SignalEngine {
    config: Arc<Config>,
    candle_buffers: Arc<RwLock<CandleBuffer>>,
    signal_generator: Arc<SignalGenerator>,
    event_bus: Option<Arc<event_bus::EventBusPublisher>>,
    repository: Option<Arc<SignalRepository>>,
}

impl SignalEngine {
    /// Initialize the signal engine with the given configuration
    pub async fn new(
        config: Config,
        event_bus: Option<Arc<event_bus::EventBusPublisher>>,
        repository: Option<Arc<SignalRepository>>,
    ) -> Result<Self> {
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
            repository,
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

        // Update candle buffer and retrieve full context
        let context = {
            let mut buffers = self.candle_buffers.write().await;
            buffers.add_candles(symbol, timeframe, candles)?;
            let mut candles_map = std::collections::HashMap::new();
            candles_map.insert(
                timeframe.to_string(),
                buffers.get_candles(symbol, timeframe).unwrap_or_default(),
            );
            crate::signals::MarketContext {
                symbol: symbol.to_string(),
                candles: candles_map,
            }
        };

        // Generate signal
        if let Some(signal) = self.signal_generator.generate(&context)? {
            info!("Generated signal for {}: {:?}", symbol, signal.direction);

            // Persist to database if configured
            if let Some(repo) = &self.repository {
                if let Err(e) = repo.save_signal(&signal).await {
                    warn!("Failed to persist signal: {}", e);
                }
            }

            // Emit to event bus if configured
            if let Some(bus) = &self.event_bus {
                let mut emitter = crate::signals::SignalEmitter::new(bus.clone());
                emitter.emit_signal(&signal).await?;
            }

            return Ok(vec![signal]);
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

    /// Get last data update timestamp
    pub async fn last_data_update(&self) -> Option<time::OffsetDateTime> {
        let buffers = self.candle_buffers.read().await;
        buffers.last_update
    }
}
