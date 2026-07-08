//! Market regime detection

pub mod detector;
pub mod types;
pub mod volatility;

pub use detector::RegimeDetector;
pub use types::{MarketRegime, RegimeType};

use crate::config::Config;
use crate::market_data::Candle;
use std::collections::HashMap;

impl RegimeDetector {
    /// Create a new regime detector
    pub fn new(config: &Config) -> Self {
        Self {
            volatility_lookback: config.volatility_lookback,
            volatility_threshold: config.volatility_percentile_threshold,
            trend_lookback: config.trend_lookback,
        }
    }

    /// Detect current market regime
    pub fn detect(&self, candles: &HashMap<String, Vec<Candle>>) -> crate::Result<MarketRegime> {
        // Use execution timeframe for regime detection
        let tf = "M15"; // Default execution timeframe

        let candles =
            candles
                .get(tf)
                .ok_or_else(|| crate::SignalEngineError::MissingTimeframe {
                    timeframe: tf.to_string(),
                })?;

        if candles.len() < self.volatility_lookback {
            return Ok(MarketRegime {
                regime_type: RegimeType::Undefined,
                confidence: 0.0,
                volatility_percentile: 0.0,
                trend_strength: 0.0,
            });
        }

        let volatility = self.calculate_volatility(candles);
        let trend_strength = self.calculate_trend_strength(candles);

        let (regime_type, confidence) = self.classify(volatility.clone(), trend_strength, candles);

        Ok(MarketRegime {
            regime_type,
            confidence,
            volatility_percentile: volatility.percentile,
            trend_strength,
        })
    }
}
