//! OHLCV candlestick representation

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// OHLCV Candlestick
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Candle {
    /// Timestamp
    pub timestamp: OffsetDateTime,
    /// Open price
    pub open: Decimal,
    /// High price
    pub high: Decimal,
    /// Low price
    pub low: Decimal,
    /// Close price
    pub close: Decimal,
    /// Volume
    pub volume: u64,
    /// Whether the candle is confirmed (closed)
    pub confirmed: bool,
}

impl Candle {
    /// Create a new candle
    pub fn new(
        timestamp: OffsetDateTime,
        open: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        volume: u64,
    ) -> Self {
        Self {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
            confirmed: false,
        }
    }

    /// Mark the candle as confirmed
    pub fn confirm(&mut self) {
        self.confirmed = true;
    }

    /// Calculate the range (high - low)
    pub fn range(&self) -> Decimal {
        self.high - self.low
    }

    /// Calculate the body size (|close - open|)
    pub fn body_size(&self) -> Decimal {
        (self.close - self.open).abs()
    }

    /// Check if candle is bullish (close > open)
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Check if candle is bearish (close < open)
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    /// Check if this is a doji (body is very small relative to range)
    pub fn is_doji(&self, threshold: Decimal) -> bool {
        self.body_size() <= self.range() * threshold
    }

    /// Get the upper wick size
    pub fn upper_wick(&self) -> Decimal {
        let body_top = self.open.max(self.close);
        self.high - body_top
    }

    /// Get the lower wick size
    pub fn lower_wick(&self) -> Decimal {
        let body_bottom = self.open.min(self.close);
        body_bottom - self.low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_candle() -> Candle {
        Candle::new(
            OffsetDateTime::now_utc(),
            Decimal::new(100, 2), // 1.00
            Decimal::new(105, 2), // 1.05
            Decimal::new(98, 2),  // 0.98
            Decimal::new(103, 2), // 1.03
            1000,
        )
    }

    #[test]
    fn test_range_calculation() {
        let candle = test_candle();
        assert_eq!(candle.range(), Decimal::new(7, 2)); // 1.05 - 0.98 = 0.07
    }

    #[test]
    fn test_bullish_bearish() {
        let bullish = Candle::new(
            OffsetDateTime::now_utc(),
            Decimal::new(1000, 3),
            Decimal::new(1050, 3),
            Decimal::new(980, 3),
            Decimal::new(1030, 3),
            100,
        );
        assert!(bullish.is_bullish());
        assert!(!bullish.is_bearish());
    }
}
