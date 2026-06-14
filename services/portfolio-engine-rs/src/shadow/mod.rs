pub mod comparison;
pub mod drift;
pub mod reporter;
pub mod shadow_storage;
pub mod statistics;
pub mod validator;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Defines a snapshot of a Shadow operation.
/// Immutable, timestamped, versioned, replayable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowSnapshot {
    pub id: uuid::Uuid,
    pub timestamp: DateTime<Utc>,
    pub version: u32,
    pub payload: serde_json::Value,
}

/// Represents an event generated during Shadow operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowEvent {
    pub event_id: uuid::Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: ShadowEventType,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShadowEventType {
    ComparisonPerformed,
    DriftDetected,
    ValidationPerformed,
    AlertGenerated,
    ReportGenerated,
}
