use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelState {
    Training,
    Shadow,
    Candidate,
    Production,
    Retired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchModel {
    pub id: Uuid,
    pub name: String,
    pub state: ModelState,
    pub version: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationModel {
    pub id: Uuid,
    pub name: String,
    pub state: ModelState,
    pub version: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationModel {
    pub id: Uuid,
    pub name: String,
    pub state: ModelState,
    pub version: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionModel {
    pub id: Uuid,
    pub name: String,
    pub state: ModelState,
    pub version: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationModel {
    pub id: Uuid,
    pub name: String,
    pub state: ModelState,
    pub version: String,
    pub created_at: OffsetDateTime,
}
