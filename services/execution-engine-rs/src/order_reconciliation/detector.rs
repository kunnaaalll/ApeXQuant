use crate::broker_connectivity::OrderState;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ReconciliationIssue {
    MissingOrder(String),
    DuplicateFill(String),
    QuantityMismatch { id: String, local: f64, broker: f64 },
    StatusMismatch { id: String, local_open: bool, broker_open: bool },
}

impl ReconciliationIssue {
    pub fn is_critical(&self) -> bool {
        match self {
            ReconciliationIssue::MissingOrder(_) => true,
            ReconciliationIssue::DuplicateFill(_) => true,
            ReconciliationIssue::QuantityMismatch { .. } => true,
            ReconciliationIssue::StatusMismatch { local_open, broker_open, .. } => {
                // If local thinks it's closed but broker thinks it's open, that's critical
                !local_open && *broker_open
            }
        }
    }
}

pub struct MismatchDetector;

impl MismatchDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect(&self, local_orders: &[OrderState], broker_orders: &[OrderState]) -> Vec<ReconciliationIssue> {
        let mut issues = Vec::new();
        let mut broker_map: HashMap<String, &OrderState> = broker_orders.iter()
            .map(|o| (o.id.clone(), o))
            .collect();

        for local in local_orders {
            if let Some(broker) = broker_map.remove(&local.id) {
                if (local.volume - broker.volume).abs() > 1e-6 {
                    issues.push(ReconciliationIssue::QuantityMismatch {
                        id: local.id.clone(),
                        local: local.volume,
                        broker: broker.volume,
                    });
                }
                if local.is_open != broker.is_open {
                    issues.push(ReconciliationIssue::StatusMismatch {
                        id: local.id.clone(),
                        local_open: local.is_open,
                        broker_open: broker.is_open,
                    });
                }
            } else {
                if local.is_open {
                    issues.push(ReconciliationIssue::MissingOrder(local.id.clone()));
                }
            }
        }

        // Remaining broker orders are missing locally
        for (_, broker) in broker_map {
            issues.push(ReconciliationIssue::MissingOrder(broker.id.clone()));
        }

        issues
    }
}
