use crate::tick::Tick;
use chrono::Utc;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TickValidationError {
    NegativeSpread,
    ZeroPrice,
    StaleTick,
    FutureTick,
}

pub struct TickProcessor {
    pub max_staleness_ms: i64,
    pub max_future_ms: i64,
}

impl Default for TickProcessor {
    fn default() -> Self {
        Self {
            max_staleness_ms: 10_000, // 10 seconds
            max_future_ms: 5_000,     // 5 seconds
        }
    }
}

impl TickProcessor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&self, mut tick: Tick) -> Result<Tick, TickValidationError> {
        // 1. Validation
        if tick.bid <= Decimal::ZERO || tick.ask <= Decimal::ZERO {
            return Err(TickValidationError::ZeroPrice);
        }

        if tick.ask < tick.bid {
            return Err(TickValidationError::NegativeSpread);
        }

        let now = Utc::now();
        let diff_ms = now.signed_duration_since(tick.timestamp).num_milliseconds();

        if diff_ms > self.max_staleness_ms {
            return Err(TickValidationError::StaleTick);
        }

        if diff_ms < -self.max_future_ms {
            return Err(TickValidationError::FutureTick);
        }

        // 2. Normalization
        tick.spread = tick.ask - tick.bid;

        Ok(tick)
    }
}
