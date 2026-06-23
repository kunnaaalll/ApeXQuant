use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EfficiencyGrade {
    Efficient,
    Normal,
    Noisy,
    Broken,
}

pub struct EfficiencyMetrics {
    pub noise_ratio: Decimal,
    pub volatility_ratio: Decimal,
    pub direction_persistence: Decimal,
    pub grade: EfficiencyGrade,
}

pub struct EfficiencyEngine;

impl EfficiencyEngine {
    pub fn evaluate(noise_ratio: Decimal, volatility_ratio: Decimal, direction_persistence: Decimal) -> Result<EfficiencyMetrics, &'static str> {
        if noise_ratio < Decimal::ZERO || volatility_ratio < Decimal::ZERO || direction_persistence < Decimal::ZERO {
            return Err("Ratios cannot be negative");
        }

        let grade = if noise_ratio > Decimal::new(80, 2) || volatility_ratio > Decimal::new(300, 2) {
            EfficiencyGrade::Broken
        } else if noise_ratio > Decimal::new(50, 2) || direction_persistence < Decimal::new(30, 2) {
            EfficiencyGrade::Noisy
        } else if noise_ratio < Decimal::new(20, 2) && direction_persistence > Decimal::new(70, 2) {
            EfficiencyGrade::Efficient
        } else {
            EfficiencyGrade::Normal
        };

        Ok(EfficiencyMetrics {
            noise_ratio,
            volatility_ratio,
            direction_persistence,
            grade,
        })
    }
}
