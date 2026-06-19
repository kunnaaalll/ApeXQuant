pub mod events;
pub mod snapshots;

use crate::regime::Regime;
use crate::session::Session;
use crate::timeframe::Timeframe;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyContextProfile {
    pub best_regime: Option<Regime>,
    pub worst_regime: Option<Regime>,
    pub best_session: Option<Session>,
    pub worst_session: Option<Session>,
    pub best_symbol: Option<String>,
    pub worst_symbol: Option<String>,
    pub best_timeframe: Option<Timeframe>,
    pub worst_timeframe: Option<Timeframe>,
    pub best_pattern: Option<String>,
    pub worst_pattern: Option<String>,
}

impl StrategyContextProfile {
    pub fn new() -> Self {
        Self {
            best_regime: None,
            worst_regime: None,
            best_session: None,
            worst_session: None,
            best_symbol: None,
            worst_symbol: None,
            best_timeframe: None,
            worst_timeframe: None,
            best_pattern: None,
            worst_pattern: None,
        }
    }
}

impl Default for StrategyContextProfile {
    fn default() -> Self {
        Self::new()
    }
}
