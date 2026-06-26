use crate::connection_supervisor::ConnectionState;

#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    BrokerConnected { broker_id: String },
    BrokerDisconnected { broker_id: String, reason: String },
    ReconciliationStarted { timestamp: u64 },
    ReconciliationCompleted { timestamp: u64, is_parity: bool },
    PositionRecovered { symbol: String, volume: f64 },
    DisasterRecoveryTriggered { timestamp: u64, reason: String },
}
