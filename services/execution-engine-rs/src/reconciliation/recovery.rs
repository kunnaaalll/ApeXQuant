use super::reconciler::{ReconciliationResult, DesyncIssue};

pub struct RecoveryEngine;

impl RecoveryEngine {
    pub fn execute_recovery(result: &ReconciliationResult) -> Result<Vec<RecoveryAction>, RecoveryError> {
        match result {
            ReconciliationResult::InSync => Ok(Vec::new()),
            ReconciliationResult::DesyncDetected { issues } => {
                let mut actions = Vec::new();
                for issue in issues {
                    match issue {
                        DesyncIssue::MissingFill { order_id } => {
                            actions.push(RecoveryAction::UpdateInternalStateToFilled { order_id: order_id.clone() });
                        },
                        DesyncIssue::PhantomBrokerOrder { order_id } => {
                            actions.push(RecoveryAction::CancelBrokerOrder { order_id: order_id.clone() });
                        },
                        DesyncIssue::MissingOnBroker { order_id } => {
                            actions.push(RecoveryAction::MarkInternalAsRejected { order_id: order_id.clone() });
                        }
                    }
                }
                Ok(actions)
            }
        }
    }
}

#[derive(Debug)]
pub enum RecoveryAction {
    UpdateInternalStateToFilled { order_id: String },
    CancelBrokerOrder { order_id: String },
    MarkInternalAsRejected { order_id: String },
}

#[derive(Debug, thiserror::Error)]
pub enum RecoveryError {
    #[error("Recovery requires manual intervention")]
    RequiresManualIntervention,
}
