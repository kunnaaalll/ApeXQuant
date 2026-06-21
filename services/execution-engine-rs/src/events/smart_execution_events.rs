use crate::execution::smart::{Priority, RoutingDecision, Urgency};
use crate::fills::FillState;
use crate::liquidity::LiquidityRegime;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmartExecutionEvent {
    RoutingDecisionMade {
        order_id: Uuid,
        urgency: Urgency,
        priority: Priority,
        decision: RoutingDecision,
    },
    PartialFillRecorded {
        order_id: Uuid,
        filled_qty: Decimal,
        remaining_qty: Decimal,
        state: FillState,
    },
    SlippageCalculated {
        order_id: Uuid,
        expected_slippage: Decimal,
        realized_slippage: Decimal,
        slippage_score: Decimal,
    },
    LiquidityEvaluated {
        order_id: Uuid,
        regime: LiquidityRegime,
    },
}
