use crate::brokers::broker::OrderState;
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ReconciliationIssue {
    MissingOrder(String),
    DuplicateFill(String),
    QuantityMismatch {
        id: String,
        local: Decimal,
        broker: Decimal,
    },
    StatusMismatch {
        id: String,
        local_open: bool,
        broker_open: bool,
    },
}

impl ReconciliationIssue {
    pub fn is_critical(&self) -> bool {
        match self {
            ReconciliationIssue::MissingOrder(_) => true,
            ReconciliationIssue::DuplicateFill(_) => true,
            ReconciliationIssue::QuantityMismatch { .. } => true,
            ReconciliationIssue::StatusMismatch {
                local_open,
                broker_open,
                ..
            } => {
                // If local thinks it's closed but broker thinks it's open, that's critical
                !local_open && *broker_open
            }
        }
    }
}

#[derive(Default)]
pub struct MismatchDetector;

impl MismatchDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect(
        &self,
        local_orders: &[OrderState],
        broker_orders: &[OrderState],
    ) -> Vec<ReconciliationIssue> {
        let mut issues = Vec::new();
        let mut broker_map: HashMap<String, &OrderState> = broker_orders
            .iter()
            .map(|o| (o.ticket.clone(), o))
            .collect();

        let tolerance = Decimal::new(1, 6);

        for local in local_orders {
            if let Some(broker) = broker_map.remove(&local.ticket) {
                if (local.volume - broker.volume).abs() > tolerance {
                    issues.push(ReconciliationIssue::QuantityMismatch {
                        id: local.ticket.clone(),
                        local: local.volume,
                        broker: broker.volume,
                    });
                }

                let local_is_open = local.status != "CLOSED" && local.status != "CANCELED";
                let broker_is_open = broker.status != "CLOSED" && broker.status != "CANCELED";

                if local_is_open != broker_is_open {
                    issues.push(ReconciliationIssue::StatusMismatch {
                        id: local.ticket.clone(),
                        local_open: local_is_open,
                        broker_open: broker_is_open,
                    });
                }
            } else {
                let local_is_open = local.status != "CLOSED" && local.status != "CANCELED";
                if local_is_open {
                    issues.push(ReconciliationIssue::MissingOrder(local.ticket.clone()));
                }
            }
        }

        // Remaining broker orders are missing locally
        for (_, broker) in broker_map {
            issues.push(ReconciliationIssue::MissingOrder(broker.ticket.clone()));
        }

        issues
    }
}
