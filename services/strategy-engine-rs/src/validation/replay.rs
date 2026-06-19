use crate::validation::events::ValidationEvent;
use crate::validation::snapshot::ValidationSnapshot;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct ReplayValidator;

impl Default for ReplayValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplayValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn verify(&self, events: &[ValidationEvent], snapshot: &ValidationSnapshot) -> bool {
        let mut reconstructed_value = Decimal::ZERO;

        for event in events {
            match event {
                ValidationEvent::ValueAdded { amount } => {
                    reconstructed_value += *amount;
                }
                ValidationEvent::ValueSubtracted { amount } => {
                    reconstructed_value -= *amount;
                }
                ValidationEvent::Multiplied { factor } => {
                    reconstructed_value *= *factor;
                }
            }
        }

        reconstructed_value == snapshot.value
    }
}
