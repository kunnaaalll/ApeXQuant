use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResearchPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResearchLifecycle {
    Queued,
    Running,
    Failed,
    Completed,
    Abandoned,
}

#[derive(Debug, Clone)]
pub struct ResearchJob {
    pub id: Uuid,
    pub priority: ResearchPriority,
    pub created_at: DateTime<Utc>,
    pub status: ResearchLifecycle,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ResearchResult {
    pub job_id: Uuid,
    pub success: bool,
    pub score: Decimal,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ResearchCandidate {
    pub id: Uuid,
    pub source_job_id: Uuid,
    pub hypothesis_id: Uuid,
    pub baseline_score: Decimal,
}

impl ResearchJob {
    pub fn new(description: String, priority: ResearchPriority) -> Self {
        Self {
            id: Uuid::new_v4(),
            priority,
            created_at: Utc::now(),
            status: ResearchLifecycle::Queued,
            description,
        }
    }

    pub fn complete(&mut self) {
        self.status = ResearchLifecycle::Completed;
    }

    pub fn fail(&mut self) {
        self.status = ResearchLifecycle::Failed;
    }
}
