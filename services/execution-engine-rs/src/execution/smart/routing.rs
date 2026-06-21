use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RoutingState {
    Primary,
    Secondary,
    Fallback,
}

impl Default for RoutingState {
    fn default() -> Self {
        Self::Primary
    }
}

impl Display for RoutingState {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RoutingState::Primary => write!(f, "Primary"),
            RoutingState::Secondary => write!(f, "Secondary"),
            RoutingState::Fallback => write!(f, "Fallback"),
        }
    }
}

/// A deterministic routing decision
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingDecision {
    pub target_venue: String,
    pub state: RoutingState,
}

impl RoutingDecision {
    pub fn new(target_venue: impl Into<String>, state: RoutingState) -> Self {
        Self {
            target_venue: target_venue.into(),
            state,
        }
    }
}
