use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HypothesisStage {
    Hypothesis,
    Experiment,
    ResearchCandidate,
    BacktestCandidate,
    ShadowCandidate,
    ProductionCandidate,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Hypothesis {
    pub id: Uuid,
    pub stage: HypothesisStage,
    pub confidence: Decimal,
    pub evidence_score: Decimal,
    pub statistical_significance: Decimal,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Hypothesis {
    pub fn new(initial_confidence: Decimal) -> Self {
        Self {
            id: Uuid::new_v4(),
            stage: HypothesisStage::Hypothesis,
            confidence: initial_confidence,
            evidence_score: Decimal::ZERO,
            statistical_significance: Decimal::ZERO,
            failure_reason: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn promote(&mut self) -> Result<(), &'static str> {
        self.stage = match self.stage {
            HypothesisStage::Hypothesis => HypothesisStage::Experiment,
            HypothesisStage::Experiment => HypothesisStage::ResearchCandidate,
            HypothesisStage::ResearchCandidate => HypothesisStage::BacktestCandidate,
            HypothesisStage::BacktestCandidate => HypothesisStage::ShadowCandidate,
            HypothesisStage::ShadowCandidate => HypothesisStage::ProductionCandidate,
            HypothesisStage::ProductionCandidate => return Err("Already at max stage"),
            HypothesisStage::Failed => return Err("Cannot promote failed hypothesis"),
        };
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn fail(&mut self, reason: String) {
        self.stage = HypothesisStage::Failed;
        self.failure_reason = Some(reason);
        self.updated_at = Utc::now();
    }
}
