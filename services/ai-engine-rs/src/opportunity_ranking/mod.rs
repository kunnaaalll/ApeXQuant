use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpportunityScore(Decimal);

impl OpportunityScore {
    pub fn new(value: Decimal) -> Result<Self, &'static str> {
        if value < Decimal::ZERO || value > Decimal::ONE_HUNDRED {
            return Err("OpportunityScore must be between 0 and 100");
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> Decimal {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfidenceScore(Decimal);

impl ConfidenceScore {
    pub fn new(value: Decimal) -> Result<Self, &'static str> {
        if value < Decimal::ZERO || value > Decimal::ONE_HUNDRED {
            return Err("ConfidenceScore must be between 0 and 100");
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> Decimal {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PriorityGrade {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RankingFactors {
    pub expectancy: Decimal,
    pub winrate: Decimal,
    pub drawdown_profile: Decimal,
    pub risk_efficiency: Decimal,
    pub regime_alignment: Decimal,
    pub execution_quality: Decimal,
    pub market_quality: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RankingTarget {
    Symbol(String),
    Session(Uuid),
    Regime(String),
    Strategy(Uuid),
    Portfolio(Uuid),
    Account(Uuid),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RankingResult {
    pub target: RankingTarget,
    pub opportunity_score: OpportunityScore,
    pub confidence_score: ConfidenceScore,
    pub priority_grade: PriorityGrade,
    pub factors: RankingFactors,
    pub evaluated_at: OffsetDateTime,
}
