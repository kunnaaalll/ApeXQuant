use super::ExposureState;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllocationEvent {
    ExposureUpdated {
        exposure: ExposureState,
        multiplier: Decimal,
    },
}
