use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolatilityState {
    Stable,
    Elevated,
    High,
    Extreme,
}

impl VolatilityState {
    pub fn evaluate(volatility_bps: Decimal) -> Result<Self, &'static str> {
        if volatility_bps < Decimal::ZERO {
            return Err("Volatility cannot be negative");
        }

        use rust_decimal::prelude::ToPrimitive;
        let vol_u64 = volatility_bps.to_u64().unwrap_or(100);

        if vol_u64 <= 5 {
            Ok(VolatilityState::Stable)
        } else if vol_u64 <= 15 {
            Ok(VolatilityState::Elevated)
        } else if vol_u64 <= 50 {
            Ok(VolatilityState::High)
        } else {
            Ok(VolatilityState::Extreme)
        }
    }
}
