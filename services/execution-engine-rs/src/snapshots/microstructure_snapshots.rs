use crate::execution_cost::total_cost::TotalExecutionCostGrade;
use crate::latency::health::LatencyState;
use crate::market::state::MarketState;
use crate::microstructure::score::MicrostructureGrade;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MicrostructureSnapshot {
    pub score: u8,
    pub grade: MicrostructureGrade,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketSnapshot {
    pub state: MarketState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatencySnapshot {
    pub total_ms: u64,
    pub state: LatencyState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionCostSnapshot {
    pub total_usd: Decimal,
    pub grade: TotalExecutionCostGrade,
}
