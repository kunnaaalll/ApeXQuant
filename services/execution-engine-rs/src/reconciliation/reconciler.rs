use super::{broker_state::BrokerState, internal_state::InternalState};
use crate::state_machine::OrderState;
use serde::{Deserialize, Serialize};

pub struct Reconciler;

impl Reconciler {
    pub fn reconcile(internal: &InternalState, broker: &BrokerState) -> ReconciliationResult {
        let mut issues = Vec::new();

        // Check for missing orders in broker (internal has it, broker doesn't)
        for int_order in &internal.tracked_orders {
            let found_on_broker = broker.active_orders.iter().find(|o| o.id == int_order.id);
            
            match int_order.state {
                OrderState::Accepted => {
                    if found_on_broker.is_none() {
                        issues.push(DesyncIssue::MissingOnBroker { order_id: int_order.id.clone() });
                    }
                },
                OrderState::Pending | OrderState::Submitted => {
                    // Expected if just submitted
                },
                _ => {}
            }
            
            if let Some(br_order) = found_on_broker {
                // If broker status is "Filled" but internal is "Accepted"
                if br_order.status == "Filled" && int_order.state == OrderState::Accepted {
                    issues.push(DesyncIssue::MissingFill { order_id: int_order.id.clone() });
                }
            }
        }

        // Check for phantom orders on broker
        for br_order in &broker.active_orders {
            let found_internal = internal.tracked_orders.iter().find(|o| o.id == br_order.id);
            if found_internal.is_none() {
                issues.push(DesyncIssue::PhantomBrokerOrder { order_id: br_order.id.clone() });
            }
        }

        if issues.is_empty() {
            ReconciliationResult::InSync
        } else {
            ReconciliationResult::DesyncDetected { issues }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DesyncIssue {
    MissingFill { order_id: String },
    PhantomBrokerOrder { order_id: String },
    MissingOnBroker { order_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReconciliationResult {
    InSync,
    DesyncDetected { issues: Vec<DesyncIssue> },
}
