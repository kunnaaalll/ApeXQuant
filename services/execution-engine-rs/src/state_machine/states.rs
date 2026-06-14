use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderState {
    /// Order created internally, waiting to be sent to the broker.
    Pending,
    /// Sent to the broker, awaiting acknowledgment.
    Submitted,
    /// Acknowledged by the broker but not yet filled.
    Accepted,
    /// Some quantity filled, but not the full amount.
    PartiallyFilled,
    /// Fully filled.
    Filled,
    /// Order is managing an active position (has SL/TP).
    Managed,
    /// Position closed or order fulfilled entirely.
    Closed,
    /// Historical record.
    Archived,
    /// Rejected by the broker.
    Rejected,
    /// Cancelled by user or system before fill.
    Cancelled,
    /// Time-in-force expired.
    Expired,
    /// Reconciler is actively fixing a desync for this order.
    Recovering,
    /// State desynced, pending reconciliation.
    Unknown,
}

impl Default for OrderState {
    fn default() -> Self {
        Self::Pending
    }
}
