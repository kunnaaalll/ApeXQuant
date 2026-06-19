use crate::regime::{Regime, RegimeGrade};
use crate::session::{Session, SessionGrade};
use crate::timeframe::{Timeframe, TimeframeGrade};
use crate::ranking::context::ContextRank;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextSnapshot {
    pub regime_states: HashMap<Regime, RegimeGrade>,
    pub session_states: HashMap<Session, SessionGrade>,
    pub symbol_states: HashMap<String, crate::symbol::SymbolGrade>,
    pub timeframe_states: HashMap<Timeframe, TimeframeGrade>,
    pub pattern_states: HashMap<String, crate::pattern::PatternGrade>,
    pub rankings: Option<ContextRank>,
}

impl ContextSnapshot {
    pub fn new() -> Self {
        Self {
            regime_states: HashMap::new(),
            session_states: HashMap::new(),
            symbol_states: HashMap::new(),
            timeframe_states: HashMap::new(),
            pattern_states: HashMap::new(),
            rankings: None,
        }
    }
}

impl Default for ContextSnapshot {
    fn default() -> Self {
        Self::new()
    }
}
