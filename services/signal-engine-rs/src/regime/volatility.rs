//! Volatility calculations and indicators

use crate::market_data::Candle;

/// Calculate True Range
pub fn true_range(candle: &Candle, prev_close: f64) -> f64 {
    let high: f64 = candle.high.to_string().parse().unwrap_or(0.0);
    let low: f64 = candle.low.to_string().parse().unwrap_or(0.0);
    let prev: f64 = prev_close;

    let tr1 = high - low;
    let tr2 = (high - prev).abs();
    let tr3 = (low - prev).abs();

    tr1.max(tr2).max(tr3)
}

/// Calculate Average True Range
pub fn atr(candles: &[Candle], period: usize) -> f64 {
    if candles.len() < period + 1 {
        return 0.0;
    }

    let mut tr_sum = 0.0;

    for i in 1..=period {
        let prev_close: f64 = candles[candles.len() - i - 1]
            .close
            .to_string()
            .parse()
            .unwrap_or(0.0);
        let tr = true_range(&candles[candles.len() - i], prev_close);
        tr_sum += tr;
    }

    tr_sum / period as f64
}

/// Calculate historical volatility (standard deviation of returns)
pub fn historical_volatility(candles: &[Candle], period: usize) -> f64 {
    if candles.len() < period + 1 {
        return 0.0;
    }

    let returns: Vec<f64> = candles
        .windows(2)
        .rev()
        .take(period)
        .map(|w| {
            let prev: f64 = w[0].close.to_string().parse().unwrap_or(1.0);
            let curr: f64 = w[1].close.to_string().parse().unwrap_or(1.0);
            if prev == 0.0 {
                0.0
            } else {
                (curr - prev) / prev
            }
        })
        .collect();

    if returns.is_empty() {
        return 0.0;
    }

    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;

    variance.sqrt()
}

/// Normalize price move by ATR
pub fn normalized_move(price_change: f64, atr: f64) -> f64 {
    if atr == 0.0 {
        0.0
    } else {
        price_change / atr
    }
}

/// Volatility regime classification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VolatilityRegime {
    /// Very low volatility
    VeryLow,
    /// Low volatility
    Low,
    /// Normal volatility
    Normal,
    /// High volatility
    High,
    /// Very high volatility
    VeryHigh,
}

/// Classify volatility level from percentile
pub fn classify_volatility(percentile: f64) -> VolatilityRegime {
    match percentile {
        p if p < 0.1 => VolatilityRegime::VeryLow,
        p if p < 0.3 => VolatilityRegime::Low,
        p if p < 0.7 => VolatilityRegime::Normal,
        p if p < 0.9 => VolatilityRegime::High,
        _ => VolatilityRegime::VeryHigh,
    }
}
