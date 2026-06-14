use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Defines the retention frequency for a snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(type_name = "snapshot_frequency", rename_all = "lowercase")]
pub enum SnapshotFrequency {
    Realtime,
    M1,
    M5,
    M15,
    H1,
    D1,
    Weekly,
    Monthly,
    Historical,
}

impl std::fmt::Display for SnapshotFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotFrequency::Realtime => write!(f, "realtime"),
            SnapshotFrequency::M1 => write!(f, "m1"),
            SnapshotFrequency::M5 => write!(f, "m5"),
            SnapshotFrequency::M15 => write!(f, "m15"),
            SnapshotFrequency::H1 => write!(f, "h1"),
            SnapshotFrequency::D1 => write!(f, "d1"),
            SnapshotFrequency::Weekly => write!(f, "weekly"),
            SnapshotFrequency::Monthly => write!(f, "monthly"),
            SnapshotFrequency::Historical => write!(f, "historical"),
        }
    }
}

/// PortfolioSnapshotWrapper encapsulates snapshots from any engine component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PortfolioSnapshotWrapper {
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

/// An immutable, append-only record of a snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotRecord {
    pub id: Uuid,
    pub aggregate_id: String,
    pub version: i64,
    pub snapshot_type: String,
    pub frequency: SnapshotFrequency,
    pub payload: PortfolioSnapshotWrapper,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

impl SnapshotRecord {
    pub fn new(
        aggregate_id: impl Into<String>,
        version: i64,
        snapshot_type: impl Into<String>,
        frequency: SnapshotFrequency,
        payload: PortfolioSnapshotWrapper,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            aggregate_id: aggregate_id.into(),
            version,
            snapshot_type: snapshot_type.into(),
            frequency,
            payload,
            timestamp: OffsetDateTime::now_utc(),
        }
    }
}
