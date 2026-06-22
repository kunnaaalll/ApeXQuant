use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventRecord {
    pub aggregate_id: Uuid,
    pub sequence_number: u64,
    pub event_type: String,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    pub payload: ExecutionEventWrapper,
    pub version: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum ExecutionEventWrapper {
    OrderEvent(serde_json::Value),
    FillEvent(serde_json::Value),
    PositionEvent(serde_json::Value),
    ExecutionRiskEvent(serde_json::Value),
    SmartExecutionEvent(serde_json::Value),
    MicrostructureEvent(serde_json::Value),
    BrokerEvent(serde_json::Value),
    ShadowEvent(serde_json::Value),
}
