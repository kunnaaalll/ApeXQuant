use crate::brokers::broker::{BrokerAdapter, BrokerError, PositionState, OrderState};
use crate::order_reconciliation::detector::MismatchDetector;

#[derive(Debug)]
pub struct RecoveryResult {
    pub positions_recovered: usize,
    pub orders_recovered: usize,
    pub is_parity_achieved: bool,
}

pub struct RecoveryEngine {
    reconciler: MismatchDetector,
}

impl RecoveryEngine {
    pub fn new() -> Self {
        Self {
            reconciler: MismatchDetector::new(),
        }
    }

    pub async fn recover<B: BrokerAdapter>(&self, broker: &B, local_orders: &[OrderState], local_positions: &[PositionState]) -> Result<RecoveryResult, BrokerError> {
        let broker_positions = broker.get_positions().await?;
        let broker_orders = broker.get_orders().await?;

        // In a real system, we would rebuild local state from broker_positions and broker_orders
        // Here we simulate the parity check.
        let order_issues = self.reconciler.detect(local_orders, &broker_orders);
        
        let mut parity = order_issues.is_empty();
        
        // Simple position parity check
        if local_positions.len() != broker_positions.len() {
            parity = false;
        }

        Ok(RecoveryResult {
            positions_recovered: broker_positions.len(),
            orders_recovered: broker_orders.len(),
            is_parity_achieved: parity,
        })
    }
}
