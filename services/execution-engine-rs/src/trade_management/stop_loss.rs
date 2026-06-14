use super::ExplainableAction;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopLossState {
    pub current_price: Decimal,
    pub initial_sl: Decimal,
    pub emergency_sl: Decimal,
    pub gap_protection_enabled: bool,
    pub is_breakeven: bool,
}

pub struct StopLossManager;

impl StopLossManager {
    /// Validation and gap protection
    pub fn evaluate_sl(state: &StopLossState, is_long: bool) -> Option<StopLossAction> {
        let price = state.current_price;
        
        // Check Emergency SL
        let emergency_breached = if is_long { price <= state.emergency_sl } else { price >= state.emergency_sl };
        if emergency_breached {
            return Some(StopLossAction {
                new_sl: price, // Execute immediately at market
                trigger_type: StopLossTriggerType::Emergency,
                explanation: format!("Emergency SL breached at {}. Immediate market close.", price),
            });
        }

        // Check Initial SL
        let initial_breached = if is_long { price <= state.initial_sl } else { price >= state.initial_sl };
        if initial_breached {
            if state.gap_protection_enabled {
                // In a real system, we'd check if this tick gapped past the SL significantly
                return Some(StopLossAction {
                    new_sl: price,
                    trigger_type: StopLossTriggerType::GapProtected,
                    explanation: format!("Initial SL breached at {}. Gap protection applied.", price),
                });
            } else {
                return Some(StopLossAction {
                    new_sl: state.initial_sl,
                    trigger_type: StopLossTriggerType::Initial,
                    explanation: format!("Initial SL breached at {}.", price),
                });
            }
        }
        
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StopLossTriggerType {
    Initial,
    Emergency,
    GapProtected,
}

pub struct StopLossAction {
    pub new_sl: Decimal,
    pub trigger_type: StopLossTriggerType,
    pub explanation: String,
}

impl ExplainableAction for StopLossAction {
    fn reason(&self) -> String {
        self.explanation.clone()
    }
}
