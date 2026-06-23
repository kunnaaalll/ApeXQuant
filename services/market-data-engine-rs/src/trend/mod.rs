use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrendDirection {
    Uptrend,
    Downtrend,
    Sideways,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrendStrength {
    Weak,
    Normal,
    Strong,
    Extreme,
}

pub struct TrendMetrics {
    pub direction: TrendDirection,
    pub strength: TrendStrength,
}

pub struct TrendEngine;

impl TrendEngine {
    pub fn evaluate(current_price: Decimal, sma_fast: Decimal, sma_slow: Decimal) -> Result<TrendMetrics, &'static str> {
        if current_price < Decimal::ZERO || sma_fast < Decimal::ZERO || sma_slow < Decimal::ZERO {
            return Err("Prices cannot be negative");
        }

        let direction = if sma_fast > sma_slow && current_price > sma_fast {
            TrendDirection::Uptrend
        } else if sma_fast < sma_slow && current_price < sma_fast {
            TrendDirection::Downtrend
        } else {
            TrendDirection::Sideways
        };

        let distance = if sma_slow.is_zero() {
            Decimal::ZERO
        } else {
            ((sma_fast - sma_slow).abs() / sma_slow) * Decimal::from(10000)
        };

        let strength = match distance {
            d if d > Decimal::from(200) => TrendStrength::Extreme,
            d if d > Decimal::from(100) => TrendStrength::Strong,
            d if d > Decimal::from(30) => TrendStrength::Normal,
            _ => TrendStrength::Weak,
        };

        Ok(TrendMetrics {
            direction,
            strength,
        })
    }
}
