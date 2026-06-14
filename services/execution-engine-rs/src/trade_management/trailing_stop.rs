use super::ExplainableAction;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrailingMode {
    ATR,
    Structure,
    Swing,
    Dynamic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailingStopState {
    pub mode: TrailingMode,
    pub current_price: Decimal,
    pub current_sl: Decimal,
    pub atr_value: Option<Decimal>,
    pub latest_structure_level: Option<Decimal>,
    pub latest_swing_level: Option<Decimal>,
    pub dynamic_step: Option<Decimal>,
}

pub struct TrailingStopEngine;

impl TrailingStopEngine {
    /// Support ATR trailing, Structure trailing, Swing trailing, Dynamic trailing.
    /// Never trail mechanically unless specific mode dictates it.
    pub fn evaluate_trail(state: &TrailingStopState, is_long: bool) -> Option<TrailingStopAction> {
        match state.mode {
            TrailingMode::ATR => {
                if let Some(atr) = state.atr_value {
                    // For example: SL = Price - 2*ATR
                    let atr_multiplier = Decimal::new(2, 0); // Configurable
                    let proposed_sl = if is_long {
                        state.current_price - (atr * atr_multiplier)
                    } else {
                        state.current_price + (atr * atr_multiplier)
                    };
                    
                    let should_trail = if is_long { proposed_sl > state.current_sl } else { proposed_sl < state.current_sl };
                    if should_trail {
                        return Some(TrailingStopAction {
                            new_sl: proposed_sl,
                            explanation: format!("ATR Trailing: New SL at {} based on ATR value {}.", proposed_sl, atr),
                        });
                    }
                }
            },
            TrailingMode::Structure => {
                if let Some(structure_level) = state.latest_structure_level {
                    let should_trail = if is_long { structure_level > state.current_sl } else { structure_level < state.current_sl };
                    if should_trail {
                        return Some(TrailingStopAction {
                            new_sl: structure_level,
                            explanation: format!("Structure Trailing: Moved SL to latest structure level {}.", structure_level),
                        });
                    }
                }
            },
            TrailingMode::Swing => {
                 if let Some(swing_level) = state.latest_swing_level {
                    let should_trail = if is_long { swing_level > state.current_sl } else { swing_level < state.current_sl };
                    if should_trail {
                        return Some(TrailingStopAction {
                            new_sl: swing_level,
                            explanation: format!("Swing Trailing: Moved SL to latest swing level {}.", swing_level),
                        });
                    }
                }
            },
            TrailingMode::Dynamic => {
                if let Some(step) = state.dynamic_step {
                    let proposed_sl = if is_long { state.current_sl + step } else { state.current_sl - step };
                    // Dynamic might trail tightly based on some internal acceleration metric
                    return Some(TrailingStopAction {
                        new_sl: proposed_sl,
                        explanation: format!("Dynamic Trailing: Accel step applied, new SL at {}.", proposed_sl),
                    });
                }
            }
        }
        
        None
    }
}

pub struct TrailingStopAction {
    pub new_sl: Decimal,
    pub explanation: String,
}

impl ExplainableAction for TrailingStopAction {
    fn reason(&self) -> String {
        self.explanation.clone()
    }
}
