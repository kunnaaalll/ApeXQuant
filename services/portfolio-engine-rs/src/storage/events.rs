use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// PortfolioEventWrapper serves as an overarching enum encapsulating
/// all domain events from across the engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PortfolioEventWrapper {
    Portfolio(serde_json::Value),
    Exposure(serde_json::Value),
    Heat(serde_json::Value),
    Allocation(serde_json::Value),
    Quality(serde_json::Value),
    Health(serde_json::Value),
    Drawdown(serde_json::Value),
    Correlation(serde_json::Value),
    Recommendation(serde_json::Value),
    Analytics(serde_json::Value),
}

/// A persistent, immutable record of an event in the system.
/// This strictly follows the append-only event sourcing philosophy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub id: Uuid,
    pub aggregate_id: String,
    pub version: i64,
    pub event_type: String,
    pub payload: PortfolioEventWrapper,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    pub metadata: serde_json::Value,
}

impl EventRecord {
    pub fn new(
        aggregate_id: impl Into<String>,
        version: i64,
        event_type: impl Into<String>,
        payload: PortfolioEventWrapper,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            aggregate_id: aggregate_id.into(),
            version,
            event_type: event_type.into(),
            payload,
            timestamp: OffsetDateTime::now_utc(),
            metadata,
        }
    }
}

/// The EventRebuilder is responsible for applying a sequential stream of events
/// to reconstruct a state deterministically.
pub struct EventRebuilder;

impl EventRebuilder {
    /// Reconstructs the aggregate state given an initial state and a list of events.
    /// This is an example interface. Actual rebuilding will be domain-specific.
    pub fn rebuild<S, F>(
        initial_state: S,
        events: &[EventRecord],
        mut apply_fn: F,
    ) -> Result<S, anyhow::Error>
    where
        F: FnMut(S, &EventRecord) -> Result<S, anyhow::Error>,
    {
        let mut state = initial_state;
        for event in events {
            state = apply_fn(state, event)?;
        }
        Ok(state)
    }
}
