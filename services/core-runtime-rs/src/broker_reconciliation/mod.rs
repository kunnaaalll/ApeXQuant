use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Order {
    pub order_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Fill {
    pub fill_id: String,
    pub price: f64,
    pub quantity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccountState {
    pub margin: f64,
    pub equity: f64,
    pub positions: HashMap<String, Position>,
    pub open_orders: HashMap<String, Order>,
    pub fills: Vec<Fill>,
}

pub struct ReconciliationEngine;

impl ReconciliationEngine {
    #[allow(clippy::float_cmp)]
    pub fn reconcile_state(
        &self,
        internal_state: &AccountState,
        broker_state: &AccountState,
    ) -> Result<(), String> {
        if internal_state.margin != broker_state.margin {
            return Err("Margin mismatch".to_string());
        }
        if internal_state.equity != broker_state.equity {
            return Err("Equity mismatch".to_string());
        }
        if internal_state.positions != broker_state.positions {
            return Err("Positions mismatch".to_string());
        }
        if internal_state.open_orders != broker_state.open_orders {
            return Err("Open orders mismatch".to_string());
        }
        if internal_state.fills != broker_state.fills {
            return Err("Fills mismatch".to_string());
        }
        Ok(())
    }
}
