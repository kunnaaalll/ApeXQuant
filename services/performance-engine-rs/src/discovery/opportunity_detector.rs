use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpportunityState {
    StrongOpportunity,
    ModerateOpportunity,
    Neutral,
    Weak,
    Avoid,
}

#[derive(Debug, Clone)]
pub struct Opportunity {
    pub state: OpportunityState,
    pub reason: String,
    pub contributing_factors: Vec<String>,
    pub confidence: Decimal,
    pub evidence: Decimal, // e.g., expectancy difference or profit factor improvement
}

pub struct OpportunityDetector {
    pub min_confidence: Decimal,
}

impl OpportunityDetector {
    pub fn new(min_confidence: Decimal) -> Self {
        Self { min_confidence }
    }

    pub fn detect(expectancy: Decimal, benchmark: Decimal, confidence: Decimal) -> Opportunity {
        let diff = expectancy - benchmark;

        let state = if confidence < rust_decimal_macros::dec!(0.5) {
            OpportunityState::Avoid
        } else if diff > rust_decimal_macros::dec!(1.0) {
            OpportunityState::StrongOpportunity
        } else if diff > rust_decimal_macros::dec!(0.5) {
            OpportunityState::ModerateOpportunity
        } else if diff > rust_decimal_macros::dec!(-0.5) {
            OpportunityState::Neutral
        } else {
            OpportunityState::Weak
        };

        Opportunity {
            state,
            reason: format!("Expectancy diff against benchmark is {}", diff),
            contributing_factors: vec!["Recent performance shift".to_string()],
            confidence,
            evidence: diff,
        }
    }
}
