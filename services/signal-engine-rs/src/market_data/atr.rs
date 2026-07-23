use crate::market_data::Candle;
use rust_decimal::Decimal;

/// Calculates Wilder's ATR from ordered OHLC candles.
///
/// The first true range uses the candle's range; subsequent ranges include
/// gaps from the previous close. A complete period is required so callers
/// never receive a fabricated volatility estimate.
pub fn calculate_atr(candles: &[Candle], period: usize) -> Option<Decimal> {
    if period == 0 || candles.len() < period {
        return None;
    }

    let mut true_ranges = Vec::with_capacity(candles.len());
    for (index, candle) in candles.iter().enumerate() {
        if candle.high < candle.low || candle.high <= Decimal::ZERO || candle.low <= Decimal::ZERO {
            return None;
        }
        let range = if index == 0 {
            candle.high - candle.low
        } else {
            let previous_close = candles[index - 1].close;
            (candle.high - candle.low)
                .max((candle.high - previous_close).abs())
                .max((candle.low - previous_close).abs())
        };
        true_ranges.push(range);
    }

    let initial_sum: Decimal = true_ranges[..period].iter().copied().sum();
    let mut atr = initial_sum / Decimal::from(period as u64);
    for true_range in true_ranges.iter().skip(period) {
        atr = ((atr * Decimal::from((period - 1) as u64)) + *true_range)
            / Decimal::from(period as u64);
    }
    Some(atr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn candle(open: i64, high: i64, low: i64, close: i64) -> Candle {
        Candle::new(
            OffsetDateTime::now_utc(),
            Decimal::from(open),
            Decimal::from(high),
            Decimal::from(low),
            Decimal::from(close),
            1,
        )
    }

    #[test]
    fn includes_gap_in_true_range() {
        let candles = vec![candle(100, 105, 95, 100), candle(110, 115, 108, 112)];
        assert_eq!(calculate_atr(&candles, 2), Some(Decimal::new(125, 1)));
    }

    #[test]
    fn requires_complete_history() {
        assert_eq!(calculate_atr(&[candle(100, 101, 99, 100)], 2), None);
    }

    #[test]
    fn volatility_changes_atr() {
        let calm = vec![candle(100, 101, 99, 100), candle(100, 101, 99, 100)];
        let volatile = vec![candle(100, 110, 90, 100), candle(100, 120, 80, 100)];
        assert!(calculate_atr(&volatile, 2) > calculate_atr(&calm, 2));
    }
}
