use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiPriority {
    Critical,
    High,
    Medium,
    Low,
    Background,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiJobStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTask {
    pub task_id: Uuid,
    pub job_id: Uuid,
    pub description: String,
    pub created_at: OffsetDateTime,
    pub priority: AiPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiJob {
    pub job_id: Uuid,
    pub status: AiJobStatus,
    pub tasks: Vec<AiTask>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiExecutionPlan {
    pub plan_id: Uuid,
    pub target_job_id: Uuid,
    pub steps: Vec<AiTask>,
    pub generated_at: OffsetDateTime,
}
