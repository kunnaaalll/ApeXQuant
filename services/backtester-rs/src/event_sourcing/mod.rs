use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryEvent {
    ResearchEvent {
        job_id: Uuid,
        status: String,
    },
    HypothesisEvent {
        hypothesis_id: Uuid,
        new_stage: String,
    },
    FeatureDiscoveryEvent {
        feature_id: Uuid,
        importance: rust_decimal::Decimal,
    },
    StrategyDiscoveryEvent {
        strategy_id: Uuid,
        family: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub payload: DiscoveryEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSnapshot {
    pub job_id: Uuid,
    pub state_data: Vec<u8>,
    pub snapshot_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverySnapshot {
    pub hypothesis_id: Uuid,
    pub state_data: Vec<u8>,
    pub snapshot_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionSnapshot {
    pub strategy_id: Uuid,
    pub parameters: Vec<u8>,
    pub snapshot_time: DateTime<Utc>,
}

pub trait EventStore {
    fn append_event(&mut self, event: EventEnvelope) -> Result<(), &'static str>;
    fn read_events(&self, aggregate_id: Uuid) -> Vec<EventEnvelope>;

    fn save_research_snapshot(&mut self, snapshot: ResearchSnapshot) -> Result<(), &'static str>;
    fn save_discovery_snapshot(&mut self, snapshot: DiscoverySnapshot) -> Result<(), &'static str>;
    fn save_evolution_snapshot(&mut self, snapshot: EvolutionSnapshot) -> Result<(), &'static str>;
}
