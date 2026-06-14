use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionEvent {
    /// Request to submit the order to the broker.
    Submit,
    /// Broker accepted the order.
    BrokerAccepted,
    /// Broker rejected the order.
    BrokerRejected { reason: String },
    /// Broker reported a partial fill.
    FillPartial { quantity: String, price: String },
    /// Broker reported a complete fill.
    FillComplete { price: String },
    /// Order was cancelled successfully.
    Cancelled,
    /// Order expired (TIF).
    Expired,
    /// Placed into active management phase.
    Manage,
    /// Reconciler detected a desync.
    DesyncDetected,
    /// Reconciler started recovery.
    RecoveryStarted,
    /// Recovery completed successfully.
    RecoveryCompleted,
    /// Position fully closed.
    PositionClosed,
    /// Archive state.
    Archive,
}
