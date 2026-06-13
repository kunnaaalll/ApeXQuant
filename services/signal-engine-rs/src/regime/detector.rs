//! Market regime detection logic

use crate::market_data::Candle;
use crate::regime::types::{MarketRegime, RegimeType};
use statrs::statistics::Statistics;

/// Detects market regimes from price action
#[derive(Debug)]
pub struct RegimeDetector {
    /// Lookback period for volatility
    pub volatility_lookback: usize,
    /// Percentile threshold for high/low volatility
    pub volatility_threshold: f64,
    /// Lookback period for trend
    pub trend_lookback: usize,
}

/// Volatility metrics
#[derive(Debug, Clone)]
pub struct VolatilityMetrics {
    /// Current volatility (ATR-like)
    pub current: f64,
    /// Percentile relative to historical
    pub percentile: f64,
    /// Historical median
    pub median: f64,
}

impl RegimeDetector {
    /// Calculate volatility metrics
    pub fn calculate_volatility(&self, candles: &[Candle]) -> VolatilityMetrics {
        if candles.len() < 2 {
            return VolatilityMetrics {
                current: 0.0,
                percentile: 0.5,
                median: 0.0,
            };
        }

        // Calculate true ranges
        let ranges: Vec<f64> = candles
            .windows(2)
            .map(|w| {
                let prev = &w[0];
                let curr = &w[1];

                let tr1 = (curr.high - curr.low).to_string().parse::<f64>().unwrap_or(0.0);
                let tr2 = (curr.high - prev.close).abs().to_string().parse::<f64>().unwrap_or(0.0);
                let tr3 = (curr.low - prev.close).abs().to_string().parse::<f64>().unwrap_or(0.0);

                tr1.max(tr2).max(tr3)
            })
            .collect();

        let lookback = self.volatility_lookback.min(ranges.len());
        let recent = &ranges[ranges.len() - lookback..];

        let current = recent.iter().sum::<f64>() / recent.len() as f64;
        let median = recent.median();

        // Calculate percentile
        let sorted_count = recent.iter().filter(|&&r| r < current).count();
        let percentile = sorted_count as f64 / recent.len() as f64;

        VolatilityMetrics {
            current,
            percentile,
            median,
        }
    }

    /// Calculate trend strength
    pub fn calculate_trend_strength(&self, candles: &[Candle]) -> f64 {
        if candles.len() < self.trend_lookback {
            return 0.0;
        }

        let lookback = self.trend_lookback.min(candles.len());
        let recent = &candles[candles.len() - lookback..];

        // Simple directional movement calculation
        let up_moves: usize = recent.windows(2).filter(|w| w[1].close > w[0].close).count();
        let down_moves: usize = recent.windows(2).filter(|w| w[1].close < w[0].close).count();

        let total = up_moves + down_moves;
        if total == 0 {
            return 0.0;
        }

        // Strength is how one-sided the moves are
        (up_moves.max(down_moves) as f64 - down_moves.min(up_moves) as f64) / total as f64
    }

    /// Classify regime from metrics
    pub fn classify(
        &self,
        volatility: VolatilityMetrics,
        trend_strength: f64,
        candles: &[Candle],
    ) -> (RegimeType, f64) {
        let is_high_vol = volatility.percentile > self.volatility_threshold;
        let is_low_vol = volatility.percentile < (1.0 - self.volatility_threshold);
        let is_trending = trend_strength > 0.6;

        // Determine price direction
        let start_price: f64 = candles[0]
            .close
            .to_string()
            .parse()
            .unwrap_or(0.0);
        let end_price: f64 = candles[candles.len() - 1]
            .close
            .to_string()
            .parse()
            .unwrap_or(0.0);
        let trending_up = end_price > start_price;

        let (regime, confidence) = match (is_high_vol, is_low_vol, is_trending, trending_up) {
            (true, _, _, _) => (RegimeType::HighVolatility, volatility.percentile),
            (_, true, false, _) => (RegimeType::LowVolatility, 1.0 - volatility.percentile),
            (_, _, true, true) => (RegimeType::TrendingUp, trend_strength),
            (_, _, true, false) => (RegimeType::TrendingDown, trend_strength),
            (_, _, false, _) => {
                // Check if potentially ranging
                let range_pct = self.calculate_range_percentage(candles);
                if range_pct > 0.7 {
                    (RegimeType::Ranging, 0.7)
                } else {
                    (RegimeType::Transition, 0.5)
                }
            }
        };

        (regime, confidence)
    }

    /// Calculate what percentage of time price stayed within a range
    fn calculate_range_percentage(&self, candles: &[Candle]) -> f64 {
        if candles.len() < 10 {
            return 0.0;
        }

        let highs: Vec<f64> = candles
            .iter()
            .map(|c| c.high.to_string().parse().unwrap_or(0.0))
            .collect();
        let lows: Vec<f64> = candles
            .iter()
            .map(|c| c.low.to_string().parse().unwrap_or(0.0))
            .collect();

        let max_high = highs.iter().copied().fold(f64::NAN, f64::max);
        let min_low = lows.iter().copied().fold(f64::NAN, f64::min);

        let range = max_high - min_low;
        if range == 0.0 {
            return 0.0;
        }

        // Calculate positions within range
        let mid = (max_high + min_low) / 2.0;
        let inner_threshold = range * 0.3; // Within 30% of center

        let within_inner: usize = candles
            .iter()
            .filter(|c| {
                let close: f64 = c.close.to_string().parse().unwrap_or(0.0);
                (close - mid).abs() < inner_threshold
            })
            .count();

        within_inner as f64 / candles.len() as f64
    }
}
