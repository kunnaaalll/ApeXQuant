use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstitutionalConfidenceScore(Decimal);

impl InstitutionalConfidenceScore {
    pub fn new(value: Decimal) -> Result<Self, &'static str> {
        if value < Decimal::ZERO || value > Decimal::ONE_HUNDRED {
            return Err("Confidence score must be between 0 and 100");
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> Decimal {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConfidenceClassification {
    VeryLow,
    Low,
    Moderate,
    High,
    InstitutionalGrade,
}

impl ConfidenceClassification {
    pub fn from_score(score: &InstitutionalConfidenceScore) -> Self {
        let val = score.value();
        if val < Decimal::from(20) {
            Self::VeryLow
        } else if val < Decimal::from(40) {
            Self::Low
        } else if val < Decimal::from(60) {
            Self::Moderate
        } else if val < Decimal::from(80) {
            Self::High
        } else {
            Self::InstitutionalGrade
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfidenceInputs {
    pub market_data_confidence: Decimal,
    pub strategy_engine_confidence: Decimal,
    pub risk_engine_confidence: Decimal,
    pub execution_engine_confidence: Decimal,
    pub portfolio_engine_confidence: Decimal,
    pub learning_engine_confidence: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfidenceEvaluation {
    pub evaluation_id: Uuid,
    pub target_id: Uuid,
    pub inputs: ConfidenceInputs,
    pub final_score: InstitutionalConfidenceScore,
    pub classification: ConfidenceClassification,
    pub evaluated_at: OffsetDateTime,
}

impl ConfidenceEvaluation {
    pub fn evaluate(target_id: Uuid, inputs: ConfidenceInputs) -> Result<Self, &'static str> {
        // Simple equal weighting for now. Sum and divide by 6.
        let total = inputs.market_data_confidence
            + inputs.strategy_engine_confidence
            + inputs.risk_engine_confidence
            + inputs.execution_engine_confidence
            + inputs.portfolio_engine_confidence
            + inputs.learning_engine_confidence;

        let avg = total / Decimal::from(6);
        let final_score = InstitutionalConfidenceScore::new(avg)?;
        let classification = ConfidenceClassification::from_score(&final_score);

        Ok(Self {
            evaluation_id: Uuid::new_v4(),
            target_id,
            inputs,
            final_score,
            classification,
            evaluated_at: OffsetDateTime::now_utc(),
        })
    }
}
