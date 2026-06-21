use crate::execution::smart::{Priority, RoutingDecision, Urgency};
use crate::fills::FillState;
use crate::liquidity::LiquidityRegime;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartExecutionSnapshot {
    pub order_id: Uuid,
    pub priority: Priority,
    pub urgency: Urgency,
    pub current_routing: Option<RoutingDecision>,
    pub fill_state: FillState,
    pub filled_qty: Decimal,
    pub remaining_qty: Decimal,
    pub latest_slippage_score: Option<Decimal>,
    pub current_liquidity_regime: LiquidityRegime,
}

impl SmartExecutionSnapshot {
    pub fn new(order_id: Uuid, initial_qty: Decimal) -> Self {
        Self {
            order_id,
            priority: Priority::default(),
            urgency: Urgency::default(),
            current_routing: None,
            fill_state: FillState::None,
            filled_qty: Decimal::ZERO,
            remaining_qty: initial_qty,
            latest_slippage_score: None,
            current_liquidity_regime: LiquidityRegime::default(),
        }
    }
}
