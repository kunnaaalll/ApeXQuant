use crate::brokers::broker::OrderState;
use super::detector::{MismatchDetector, ReconciliationIssue};

#[derive(Debug, Clone, PartialEq)]
pub enum ReconciliationState {
    ExactMatch,
    Warning(Vec<ReconciliationIssue>),
    CriticalMismatch(Vec<ReconciliationIssue>),
}

#[derive(Debug)]
pub struct ReconciliationResult {
    pub state: ReconciliationState,
}

pub struct ReconciliationEngine {
    detector: MismatchDetector,
}

impl ReconciliationEngine {
    pub fn new() -> Self {
        Self {
            detector: MismatchDetector::new(),
        }
    }

    pub fn reconcile(&self, local_orders: &[OrderState], broker_orders: &[OrderState]) -> ReconciliationResult {
        let issues = self.detector.detect(local_orders, broker_orders);
        
        let state = if issues.is_empty() {
            ReconciliationState::ExactMatch
        } else {
            let has_critical = issues.iter().any(|i| i.is_critical());
            if has_critical {
                ReconciliationState::CriticalMismatch(issues)
            } else {
                ReconciliationState::Warning(issues)
            }
        };

        ReconciliationResult { state }
    }
}
