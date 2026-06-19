use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClusterType {
    RiskOn,
    RiskOff,
    Momentum,
    Breakout,
    TrendFollowing,
    Scalping,
    Swing,
    MeanReversion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterState {
    pub active_cluster: ClusterType,
    pub confidence: Decimal,
}

impl ClusterState {
    pub fn new() -> Self {
        Self {
            active_cluster: ClusterType::RiskOff,
            confidence: rust_decimal_macros::dec!(0.0),
        }
    }
}

impl Default for ClusterState {
    fn default() -> Self {
        Self::new()
    }
}
