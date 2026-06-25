use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::features::{FeatureVector, FeatureSnapshot};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureEvent {
    pub symbol: String,
    pub window: String,
    pub features: FeatureVector,
    pub timestamp: DateTime<Utc>,
}

// Re-export FeatureSnapshot as it's defined in features::types
// but often queried as part of Event Sourcing Snapshots
// pub use crate::features::FeatureSnapshot;
