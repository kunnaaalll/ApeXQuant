use crate::shadow::validator::GoLiveState;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowSnapshot {
    pub match_percentage: Decimal,
    pub consecutive_exact_matches: u64,
    pub go_live_state: GoLiveState,
}
