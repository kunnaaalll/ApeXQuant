use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationState {
    IncreaseExposure,
    MaintainExposure,
    ReduceExposure,
    AvoidExposure,
}

#[derive(Debug, Clone)]
pub struct AllocationRecommendation {
    pub state: AllocationState,
    pub reason: String,
    pub largest_contributor: String,
    pub largest_penalty: String,
    pub confidence: Decimal,
    pub historical_evidence: Decimal, // E.g., past expected value
}

#[derive(Debug, Clone)]
pub struct AllocationEngine {
    pub min_increase_confidence: Decimal,
    pub max_avoid_drawdown: Decimal,
}

impl AllocationEngine {
    pub fn new(min_increase_confidence: Decimal, max_avoid_drawdown: Decimal) -> Self {
        Self {
            min_increase_confidence,
            max_avoid_drawdown,
        }
    }

    pub fn evaluate(
        &self,
        confidence: Decimal,
        current_drawdown: Decimal,
        edge_score: Decimal,
    ) -> AllocationState {
        if current_drawdown >= self.max_avoid_drawdown {
            return AllocationState::AvoidExposure;
        }

        if edge_score < rust_decimal_macros::dec!(0.0) {
            return AllocationState::ReduceExposure;
        }

        if confidence >= self.min_increase_confidence && edge_score > rust_decimal_macros::dec!(0.5)
        {
            AllocationState::IncreaseExposure
        } else {
            AllocationState::MaintainExposure
        }
    }
}
