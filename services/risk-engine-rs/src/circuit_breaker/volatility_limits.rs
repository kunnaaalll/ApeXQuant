use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VolatilityState {
    Stable = 0,
    Elevated = 1,
    Danger = 2,
    Extreme = 3,
    Frozen = 4,
}

#[derive(Debug, Clone)]
pub struct VolatilityLimitAssessment {
    pub volatility_multiplier: Decimal,
    pub state: VolatilityState,
}

impl VolatilityLimitAssessment {
    pub fn new(mut volatility_multiplier: Decimal) -> Self {
        // Clamp volatility multiplier: never negative, max out at 10.0x to avoid unbounded risk
        if volatility_multiplier.is_sign_negative() {
            volatility_multiplier = Decimal::ZERO;
        }
        let max_multiplier = Decimal::new(100, 1); // 10.0
        if volatility_multiplier > max_multiplier {
            volatility_multiplier = max_multiplier;
        }

        let mut assessment = Self {
            volatility_multiplier,
            state: VolatilityState::Stable,
        };
        assessment.update_state();
        assessment
    }

    fn update_state(&mut self) {
        self.state = if self.volatility_multiplier >= Decimal::new(50, 1) { // >= 5.0x
            VolatilityState::Frozen
        } else if self.volatility_multiplier >= Decimal::new(40, 1) { // >= 4.0x
            VolatilityState::Extreme
        } else if self.volatility_multiplier >= Decimal::new(30, 1) { // >= 3.0x
            VolatilityState::Danger
        } else if self.volatility_multiplier >= Decimal::new(20, 1) { // >= 2.0x
            VolatilityState::Elevated
        } else {
            VolatilityState::Stable
        };
    }
}
