use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StrategyState {
    Elite,
    Strong,
    #[default]
    Normal,
    Weak,
    Dying,
    Retired,
}

impl fmt::Display for StrategyState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrategyState::Elite => write!(f, "Elite"),
            StrategyState::Strong => write!(f, "Strong"),
            StrategyState::Normal => write!(f, "Normal"),
            StrategyState::Weak => write!(f, "Weak"),
            StrategyState::Dying => write!(f, "Dying"),
            StrategyState::Retired => write!(f, "Retired"),
        }
    }
}
