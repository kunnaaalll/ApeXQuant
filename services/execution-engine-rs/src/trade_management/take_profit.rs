use super::ExplainableAction;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeProfitState {
    pub current_price: Decimal,
    pub targets: Vec<TakeProfitTarget>,
    pub dynamic_tp_enabled: bool,
    pub dynamic_trigger_price: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeProfitTarget {
    pub id: u32,
    pub price: Decimal,
    pub volume_percentage: Decimal,
    pub filled: bool,
}

pub struct TakeProfitManager;

impl TakeProfitManager {
    pub fn evaluate_tp(state: &TakeProfitState, is_long: bool) -> Option<TakeProfitAction> {
        let price = state.current_price;
        
        // Check dynamic TP first
        if state.dynamic_tp_enabled {
            if let Some(trigger) = state.dynamic_trigger_price {
                let dynamic_hit = if is_long { price >= trigger } else { price <= trigger };
                if dynamic_hit {
                    return Some(TakeProfitAction {
                        target_id: 0,
                        volume_percentage: Decimal::new(100, 0), // Close all
                        explanation: format!("Dynamic TP condition met at {}. Closing position.", price),
                    });
                }
            }
        }

        // Check multiple fixed targets
        for target in &state.targets {
            if !target.filled {
                let hit = if is_long { price >= target.price } else { price <= target.price };
                if hit {
                    return Some(TakeProfitAction {
                        target_id: target.id,
                        volume_percentage: target.volume_percentage,
                        explanation: format!("Fixed TP Target {} hit at {}. Closing {}%.", target.id, target.price, target.volume_percentage),
                    });
                }
            }
        }
        
        None
    }
}

pub struct TakeProfitAction {
    pub target_id: u32,
    pub volume_percentage: Decimal,
    pub explanation: String,
}

impl ExplainableAction for TakeProfitAction {
    fn reason(&self) -> String {
        self.explanation.clone()
    }
}
