use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftMeasurement {
    pub metric_name: String,
    pub legacy_value: Decimal,
    pub rust_value: Decimal,
    pub absolute_drift: Decimal,
    pub relative_drift: Decimal,
}

impl DriftMeasurement {
    pub fn new(metric_name: String, legacy_value: Decimal, rust_value: Decimal) -> Self {
        let absolute_drift = (rust_value - legacy_value).abs();

        let relative_drift = if legacy_value == Decimal::ZERO {
            if rust_value == Decimal::ZERO {
                Decimal::ZERO
            } else {
                Decimal::new(100, 0) // 100% drift if legacy is 0 and rust is not
            }
        } else {
            (absolute_drift / legacy_value.abs()) * Decimal::new(100, 0)
        };

        Self {
            metric_name,
            legacy_value,
            rust_value,
            absolute_drift,
            relative_drift,
        }
    }

    pub fn has_significant_drift(&self, threshold: Decimal) -> bool {
        self.absolute_drift > threshold
    }
}
