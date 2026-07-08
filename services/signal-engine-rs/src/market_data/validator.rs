//! Market data quality validation

use crate::market_data::Candle;
use crate::SignalEngineError;

/// Validates market data quality
#[derive(Debug)]
pub struct DataValidator;

impl DataValidator {
    /// Validate a single candle
    pub fn validate_candle(candle: &Candle) -> Result<(), SignalEngineError> {
        // Check price ordering
        if candle.low > candle.high {
            return Err(SignalEngineError::validation(
                "Low price cannot be greater than high price",
            ));
        }

        if candle.open > candle.high || candle.open < candle.low {
            return Err(SignalEngineError::validation(
                "Open price outside high/low range",
            ));
        }

        if candle.close > candle.high || candle.close < candle.low {
            return Err(SignalEngineError::validation(
                "Close price outside high/low range",
            ));
        }

        // Check positive prices
        if candle.open <= rust_decimal::Decimal::ZERO
            || candle.high <= rust_decimal::Decimal::ZERO
            || candle.low <= rust_decimal::Decimal::ZERO
            || candle.close <= rust_decimal::Decimal::ZERO
        {
            return Err(SignalEngineError::validation("Prices must be positive"));
        }

        Ok(())
    }

    /// Validate a series of candles
    pub fn validate_series(candles: &[Candle]) -> Result<(), SignalEngineError> {
        if candles.is_empty() {
            return Err(SignalEngineError::validation("Empty candle series"));
        }

        if candles.len() < 2 {
            return Ok(());
        }

        // Check for chronological ordering
        for i in 1..candles.len() {
            if candles[i].timestamp <= candles[i - 1].timestamp {
                return Err(SignalEngineError::validation(format!(
                    "Candles out of order at index {}: {} <= {}",
                    i,
                    candles[i].timestamp,
                    candles[i - 1].timestamp
                )));
            }

            // Check for gaps (configurable per timeframe ideally, hardcoded threshold for now)
            let gap = candles[i].timestamp - candles[i - 1].timestamp;
            // A basic check: if gap is larger than a weekend (approx 48 hours) + 1 day
            if gap.whole_seconds() > 3 * 24 * 3600 {
                return Err(SignalEngineError::validation(format!(
                    "Unusually large gap detected between {} and {} ({} seconds)",
                    candles[i - 1].timestamp,
                    candles[i].timestamp,
                    gap.whole_seconds()
                )));
            }
        }

        // Validate each candle
        for (i, candle) in candles.iter().enumerate() {
            Self::validate_candle(candle).map_err(|e| {
                SignalEngineError::validation(format!("Invalid candle at index {}: {}", i, e))
            })?;
        }

        Ok(())
    }

    /// Check for suspicious outliers
    pub fn check_outliers(candles: &[Candle], atr_multiplier: f64) -> Vec<usize> {
        let mut outliers = Vec::new();

        if candles.len() < 20 {
            return outliers;
        }

        // Calculate ATR (simple version)
        let ranges: Vec<f64> = candles
            .iter()
            .map(|c| (c.high - c.low).to_string().parse::<f64>().unwrap_or(0.0))
            .collect();

        let atr_sum: f64 = ranges[ranges.len() - 14..].iter().sum();
        let atr = atr_sum / 14.0;

        // Find outliers
        for (i, candle) in candles.iter().enumerate().skip(1) {
            let range = (candle.high - candle.low)
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0);

            if range > atr * atr_multiplier {
                outliers.push(i);
            }
        }

        outliers
    }

    /// Check data freshness
    pub fn check_freshness(
        last_candle: &Candle,
        max_age_seconds: i64,
    ) -> Result<(), SignalEngineError> {
        let now = time::OffsetDateTime::now_utc();
        let age = now - last_candle.timestamp;

        if age.whole_seconds() > max_age_seconds {
            return Err(SignalEngineError::validation(format!(
                "Data is stale: {} seconds old",
                age.whole_seconds()
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn valid_candle() -> Candle {
        Candle::new(
            OffsetDateTime::now_utc(),
            rust_decimal::Decimal::new(100, 2),
            rust_decimal::Decimal::new(105, 2),
            rust_decimal::Decimal::new(98, 2),
            rust_decimal::Decimal::new(103, 2),
            1000,
        )
    }

    #[test]
    fn test_valid_candle() {
        let candle = valid_candle();
        assert!(DataValidator::validate_candle(&candle).is_ok());
    }

    #[test]
    fn test_invalid_low_high() {
        let mut candle = valid_candle();
        candle.low = rust_decimal::Decimal::new(110, 2); // Higher than high
        assert!(DataValidator::validate_candle(&candle).is_err());
    }

    #[test]
    fn test_negative_prices() {
        let mut candle = valid_candle();
        candle.close = rust_decimal::Decimal::ZERO;
        assert!(DataValidator::validate_candle(&candle).is_err());
    }
}
