use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MomentumGrade {
    Explosive,
    Strong,
    Normal,
    Weak,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMetrics {
    pub strength: Decimal,
    pub velocity: Decimal,
    pub decay: Decimal,
    pub acceleration: Decimal,
    pub divergence_detected: bool,
    pub grade: MomentumGrade,
}

#[derive(Debug, Clone)]
pub struct MomentumEngine {
    window: usize,
    prices: Vec<Decimal>,
    velocities: Vec<Decimal>,
}

impl MomentumEngine {
    pub fn new(window: usize) -> Self {
        Self {
            window,
            prices: Vec::with_capacity(window + 1),
            velocities: Vec::with_capacity(window),
        }
    }

    pub fn update(&mut self, price: Decimal) -> Result<MomentumMetrics, &'static str> {
        self.prices.push(price);
        if self.prices.len() > self.window + 1 {
            self.prices.remove(0);
        }

        let mut velocity = Decimal::ZERO;
        if self.prices.len() >= 2 {
            let last = self.prices[self.prices.len() - 1];
            let prev = self.prices[self.prices.len() - 2];
            if !prev.is_zero() {
                velocity = (last - prev) / prev * Decimal::from(10000); // bps
            }
        }

        self.velocities.push(velocity);
        if self.velocities.len() > self.window {
            self.velocities.remove(0);
        }

        let mut acceleration = Decimal::ZERO;
        if self.velocities.len() >= 2 {
            let last_v = self.velocities[self.velocities.len() - 1];
            let prev_v = self.velocities[self.velocities.len() - 2];
            acceleration = last_v - prev_v;
        }

        let strength = if self.velocities.is_empty() {
            Decimal::ZERO
        } else {
            let sum: Decimal = self.velocities.iter().sum();
            sum / Decimal::from(self.velocities.len())
        };

        let decay = if (strength.is_sign_positive() && acceleration.is_sign_negative()) || (strength.is_sign_negative() && acceleration.is_sign_positive()) {
            acceleration.abs()
        } else {
            Decimal::ZERO
        };

        // Simple divergence proxy: price makes higher high, but momentum strength doesn't (we would need history of swing highs to do it perfectly, but for now we'll do a simple local check)
        let divergence_detected = false; // To be refined with Structure Engine inputs

        let grade = match strength {
            s if s > Decimal::from(100) => MomentumGrade::Explosive,
            s if s > Decimal::from(50) => MomentumGrade::Strong,
            s if s > Decimal::from(10) => MomentumGrade::Normal,
            s if s > Decimal::ZERO => MomentumGrade::Weak,
            _ => MomentumGrade::Negative,
        };

        Ok(MomentumMetrics {
            strength,
            velocity,
            decay,
            acceleration,
            divergence_detected,
            grade,
        })
    }
}
