use super::ExplainableAction;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialCloseState {
    pub current_exposure: Decimal,
    pub entry_price: Decimal,
    pub current_price: Decimal,
    pub realized_pnl: Decimal,
}

pub struct PartialCloseEngine;

impl PartialCloseEngine {
    /// Support custom percentages. Track realized PnL, remaining exposure.
    pub fn calculate_partial_close(state: &PartialCloseState, percentage: Decimal, is_long: bool) -> PartialCloseAction {
        let close_qty = state.current_exposure * (percentage / Decimal::new(100, 0));
        let remaining_exposure = state.current_exposure - close_qty;
        
        let price_diff = if is_long { state.current_price - state.entry_price } else { state.entry_price - state.current_price };
        let pnl_addition = close_qty * price_diff;
        let new_realized_pnl = state.realized_pnl + pnl_addition;

        PartialCloseAction {
            percentage,
            close_qty,
            remaining_exposure,
            new_realized_pnl,
            explanation: format!(
                "Executing partial close of {}% ({} units). New realized PnL: {}. Remaining exposure: {}.", 
                percentage, close_qty, new_realized_pnl, remaining_exposure
            ),
        }
    }
}

pub struct PartialCloseAction {
    pub percentage: Decimal,
    pub close_qty: Decimal,
    pub remaining_exposure: Decimal,
    pub new_realized_pnl: Decimal,
    pub explanation: String,
}

impl ExplainableAction for PartialCloseAction {
    fn reason(&self) -> String {
        self.explanation.clone()
    }
}
