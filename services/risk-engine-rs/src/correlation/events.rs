use super::{clusters::CorrelationCluster, hidden_leverage::HiddenLeverageState, CorrelationSeverity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrelationRiskEvent {
    CorrelationUpdated {
        dimension: String,
        entity_a: String,
        entity_b: String,
    },
    ClusterDetected {
        cluster: CorrelationCluster,
    },
    HiddenLeverageChanged {
        old_state: HiddenLeverageState,
        new_state: HiddenLeverageState,
    },
    SeverityChanged {
        old_severity: CorrelationSeverity,
        new_severity: CorrelationSeverity,
        reason: String,
    },
    WindowRolled {
        window_type: super::windows::CorrelationWindowType,
    },
}
