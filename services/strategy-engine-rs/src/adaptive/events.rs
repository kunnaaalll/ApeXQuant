use super::{WeightState, WeightType};
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdaptiveEvent {
    WeightUpdated {
        weight_type: WeightType,
        old_weight: Decimal,
        new_weight: Decimal,
        state: WeightState,
    },
}
