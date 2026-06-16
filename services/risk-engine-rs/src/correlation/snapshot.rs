use super::{
    clusters::CorrelationCluster, correlation_matrix::CorrelationMatrix, events::CorrelationRiskEvent,
    hidden_leverage::HiddenLeverageState,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrelationRiskSnapshot {
    pub version: u32,
    pub timestamp: i64,
    pub event: CorrelationRiskEvent,
    pub matrix_state: CorrelationMatrix,
    pub hidden_leverage_state: HiddenLeverageState,
    pub clusters: Vec<CorrelationCluster>,
}

impl CorrelationRiskSnapshot {
    pub fn new(
        version: u32,
        timestamp: i64,
        event: CorrelationRiskEvent,
        matrix_state: CorrelationMatrix,
        hidden_leverage_state: HiddenLeverageState,
        clusters: Vec<CorrelationCluster>,
    ) -> Self {
        Self {
            version,
            timestamp,
            event,
            matrix_state,
            hidden_leverage_state,
            clusters,
        }
    }
}
