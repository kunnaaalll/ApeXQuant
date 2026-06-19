use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExposureState {
    IncreaseExposure,
    SlightIncrease,
    Maintain,
    ReduceExposure,
    Pause,
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllocationState {
    pub exposure: ExposureState,
    pub multiplier: Decimal,
}

impl AllocationState {
    pub fn new() -> Self {
        Self {
            exposure: ExposureState::Maintain,
            multiplier: rust_decimal_macros::dec!(1.00),
        }
    }
}

impl Default for AllocationState {
    fn default() -> Self {
        Self::new()
    }
}
