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
        let tf_candles = ["M15", "M5", "M1"]
            .iter()
            .find_map(|tf| candles.get(*tf))
            .or_else(|| candles.values().next());

        let candles = match tf_candles {
            Some(c) => c,
            None => {
                return Ok(MarketRegime {
                    regime_type: RegimeType::Undefined,
                    confidence: 0.0,
                    volatility_percentile: 0.0,
                    trend_strength: 0.0,
                })
            }
        };

        if candles.len() < self.volatility_lookback {
            return Ok(MarketRegime {
                regime_type: RegimeType::Ranging,
                confidence: 0.5,
                volatility_percentile: 0.5,
                trend_strength: 0.5,
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
