use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VelocityState {
    Accelerating,
    Stable,
    Decelerating,
    Reversing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VelocityType {
    Edge,
    Expectancy,
    Stability,
    Confidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VelocityEngine {
    previous_velocity: Decimal,
    state: VelocityState,
}

impl VelocityEngine {
    pub fn new() -> Self {
        Self {
            previous_velocity: dec!(0.0),
            state: VelocityState::Stable,
        }
    }

    pub fn update(&mut self, current_velocity: Decimal) -> VelocityState {
        let acceleration = current_velocity - self.previous_velocity;
        
        // Use 0 for comparisons
        let zero = dec!(0.0);
        let sig_accel = dec!(0.01);
        let sig_decel = dec!(-0.01);

        self.state = if (current_velocity > zero && self.previous_velocity < zero) || (current_velocity < zero && self.previous_velocity > zero) {
            VelocityState::Reversing
        } else if (current_velocity > zero && acceleration > sig_accel) || (current_velocity < zero && acceleration < sig_decel) {
            VelocityState::Accelerating
        } else if (current_velocity > zero && acceleration < sig_decel) || (current_velocity < zero && acceleration > sig_accel) {
            VelocityState::Decelerating
        } else {
            VelocityState::Stable
        };

        self.previous_velocity = current_velocity;
        self.state
    }

    pub fn state(&self) -> VelocityState {
        self.state
    }
}

impl Default for VelocityEngine {
    fn default() -> Self {
        Self::new()
    }
}
