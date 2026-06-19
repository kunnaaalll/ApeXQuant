use super::{AllocationState, ExposureState};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllocationEngine {
    state: AllocationState,
}

impl AllocationEngine {
    pub fn new() -> Self {
        Self {
            state: AllocationState::new(),
        }
    }

    pub fn compute(
        &mut self,
        strategy_health: Decimal, // 0.0 to 100.0 expected
        confidence: Decimal,      // 0.0 to 100.0
        degradation: Decimal,     // Higher is worse
        drawdown: Decimal,        // absolute decimal
        sample_quality: Decimal,  // 0.0 to 100.0
    ) {
        // Compute base safety
        let safety = strategy_health + confidence + sample_quality;
        let risk = degradation + drawdown;

        // Bounding multiplier
        let mut target_multiplier = if risk > dec!(50.0) {
            dec!(0.25)
        } else if risk > dec!(20.0) {
            dec!(0.50)
        } else if safety >= dec!(250.0) {
            dec!(2.00)
        } else if safety >= dec!(200.0) {
            dec!(1.50)
        } else if safety >= dec!(150.0) {
            dec!(1.10)
        } else {
            dec!(1.00)
        };

        // Enforce exact bounds
        target_multiplier = Self::clamp_multiplier(target_multiplier);

        self.state.multiplier = target_multiplier;

        self.state.exposure = if target_multiplier == dec!(0.25) {
            ExposureState::Block
        } else if target_multiplier < dec!(0.50) {
            ExposureState::Pause
        } else if target_multiplier < dec!(1.00) {
            ExposureState::ReduceExposure
        } else if target_multiplier == dec!(1.00) {
            ExposureState::Maintain
        } else if target_multiplier <= dec!(1.50) {
            ExposureState::SlightIncrease
        } else {
            ExposureState::IncreaseExposure
        };
    }

    pub fn state(&self) -> &AllocationState {
        &self.state
    }

    fn clamp_multiplier(val: Decimal) -> Decimal {
        let min = dec!(0.25);
        let max = dec!(2.00);
        if val < min {
            min
        } else if val > max {
            max
        } else {
            val
        }
    }
}

impl Default for AllocationEngine {
    fn default() -> Self {
        Self::new()
    }
}
