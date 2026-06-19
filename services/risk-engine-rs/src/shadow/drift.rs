use rust_decimal::Decimal;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftSeverity {
    Normal,
    Elevated,
    High,
    Critical,
}

pub struct DriftEngine;

impl DriftEngine {
    pub fn measure_absolute(legacy: Decimal, rust: Decimal) -> Decimal {
        (legacy - rust).abs()
    }

    pub fn measure_relative(legacy: Decimal, rust: Decimal) -> Decimal {
        if legacy.is_zero() {
            if rust.is_zero() {
                return Decimal::ZERO;
            } else {
                return Decimal::ONE; // Bound relative drift to 1.0 (100%) max when baseline is 0
            }
        }

        let diff = (legacy - rust).abs();
        let relative = diff / legacy.abs();

        if relative > Decimal::ONE {
            Decimal::ONE
        } else {
            relative
        }
    }

    pub fn classify(absolute_drift: Decimal, relative_drift: Decimal) -> DriftSeverity {
        let abs_critical = Decimal::new(100, 0); // 100.0
        let abs_high = Decimal::new(10, 0); // 10.0
        let abs_elevated = Decimal::new(1, 0); // 1.0

        let rel_critical = Decimal::new(5, 1); // 0.5
        let rel_high = Decimal::new(1, 1); // 0.1
        let rel_elevated = Decimal::new(1, 2); // 0.01

        if absolute_drift >= abs_critical || relative_drift >= rel_critical {
            DriftSeverity::Critical
        } else if absolute_drift >= abs_high || relative_drift >= rel_high {
            DriftSeverity::High
        } else if absolute_drift >= abs_elevated || relative_drift >= rel_elevated {
            DriftSeverity::Elevated
        } else {
            DriftSeverity::Normal
        }
    }
}
