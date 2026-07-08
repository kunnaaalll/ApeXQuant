use crate::execution_cost::total_cost::TotalExecutionCostGrade;
use crate::latency::health::LatencyState;
use crate::market::state::MarketState;
use crate::microstructure::score::MicrostructureGrade;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MicrostructureEvent {
    ScoreUpdated {
        score: u8,
        grade: MicrostructureGrade,
    },
    SpreadUpdated {
        absolute: Decimal,
        relative: Decimal,
    },
    DepthUpdated {
        cumulative: Decimal,
    },
    ImbalanceUpdated {
        score: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarketStateEvent {
    StateChanged { from: MarketState, to: MarketState },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LatencyEvent {
    LatencyUpdated { total_ms: u64, state: LatencyState },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionCostEvent {
    CostCalculated {
        total_usd: Decimal,
        grade: TotalExecutionCostGrade,
    },
}
