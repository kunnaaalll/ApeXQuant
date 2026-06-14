use super::ExplainableAction;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakevenState {
    pub entry_price: Decimal,
    pub current_price: Decimal,
    pub min_rr_reached: bool,
    pub structure_supports: bool,
    pub volatility_permits: bool,
    pub already_at_breakeven: bool,
}

pub struct BreakevenEngine;

impl BreakevenEngine {
    /// Move SL to breakeven only when minimum RR reached, market structure supports it, and volatility permits.
    pub fn evaluate_breakeven(state: &BreakevenState) -> Option<BreakevenAction> {
        if state.already_at_breakeven {
            return None;
        }

        if state.min_rr_reached && state.structure_supports && state.volatility_permits {
            return Some(BreakevenAction {
                new_sl_price: state.entry_price,
                explanation: format!("Moved to breakeven at {}. Min RR, structure, and volatility conditions met.", state.entry_price),
            });
        }
        
        None
    }
}

pub struct BreakevenAction {
    pub new_sl_price: Decimal,
    pub explanation: String,
}

impl ExplainableAction for BreakevenAction {
    fn reason(&self) -> String {
        self.explanation.clone()
    }
}
