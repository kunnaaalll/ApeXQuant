use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchPriorityScore(Decimal);

impl ResearchPriorityScore {
    pub fn new(value: Decimal) -> Result<Self, &'static str> {
        if value < Decimal::ZERO || value > Decimal::ONE_HUNDRED {
            return Err("Research priority score must be between 0 and 100");
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> Decimal {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResearchState {
    Ignore,
    Queue,
    Research,
    ImmediateResearch,
}

impl ResearchState {
    pub fn from_score(score: &ResearchPriorityScore) -> Self {
        let val = score.value();
        if val < Decimal::from(25) {
            Self::Ignore
        } else if val < Decimal::from(50) {
            Self::Queue
        } else if val < Decimal::from(75) {
            Self::Research
        } else {
            Self::ImmediateResearch
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvaluationTrigger {
    UnderexploredSymbol(String),
    DeterioratingStrategy(Uuid),
    MissingRegime(String),
    EmergingOpportunity(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchRequest {
    pub request_id: Uuid,
    pub trigger: EvaluationTrigger,
    pub priority_score: ResearchPriorityScore,
    pub state: ResearchState,
    pub generated_at: OffsetDateTime,
}

impl ResearchRequest {
    pub fn evaluate(trigger: EvaluationTrigger, base_score: Decimal) -> Result<Self, &'static str> {
        let priority_score = ResearchPriorityScore::new(base_score)?;
        let state = ResearchState::from_score(&priority_score);

        Ok(Self {
            request_id: Uuid::new_v4(),
            trigger,
            priority_score,
            state,
            generated_at: OffsetDateTime::now_utc(),
        })
    }
}
