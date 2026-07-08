use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use super::circuit_breaker::ExecutionProtectionState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillGrade {
    Elite,
    Good,
    Normal,
    Poor,
    Broken,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FillQualityGuards {
    pub partial_fills: u32,
    pub missed_fills: u32,
    pub fill_ratio: Decimal,
    pub total_orders: u32,
}

impl FillQualityGuards {
    pub fn new(partial_fills: u32, missed_fills: u32, total_orders: u32) -> Self {
        let fill_ratio = if total_orders == 0 {
            Decimal::ONE
        } else {
            let successful = total_orders.saturating_sub(missed_fills);
            Decimal::from(successful) / Decimal::from(total_orders)
        };

        Self {
            partial_fills,
            missed_fills,
            fill_ratio,
            total_orders,
        }
    }

    pub fn get_grade(&self) -> FillGrade {
        if self.total_orders < 10 {
            return FillGrade::Normal; // Not enough data
        }

        if self.fill_ratio >= dec!(0.99) {
            FillGrade::Elite
        } else if self.fill_ratio >= dec!(0.95) {
            FillGrade::Good
        } else if self.fill_ratio >= dec!(0.90) {
            FillGrade::Normal
        } else if self.fill_ratio >= dec!(0.80) {
            FillGrade::Poor
        } else {
            FillGrade::Broken
        }
    }

    pub fn get_state(&self) -> ExecutionProtectionState {
        match self.get_grade() {
            FillGrade::Elite | FillGrade::Good | FillGrade::Normal => {
                ExecutionProtectionState::Normal
            }
            FillGrade::Poor => ExecutionProtectionState::Restricted,
            FillGrade::Broken => ExecutionProtectionState::Critical,
        }
    }
}
