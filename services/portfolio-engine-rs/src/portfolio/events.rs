use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::state::RecoveryState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PortfolioEvent {
    PositionOpened {
        position_id: Uuid,
        margin_used: Decimal,
        exposure: Decimal,
    },
    PositionClosed {
        position_id: Uuid,
        realized_pnl: Decimal,
        margin_released: Decimal,
        exposure_released: Decimal,
    },
    PartialClose {
        position_id: Uuid,
        realized_pnl: Decimal,
        margin_released: Decimal,
        exposure_released: Decimal,
    },
    PnlUpdate {
        position_id: Uuid,
        pnl_delta: Decimal,
    },
    BalanceChange {
        amount: Decimal,
        reason: String,
    },
    Deposit {
        amount: Decimal,
    },
    Withdrawal {
        amount: Decimal,
    },
    MarginChange {
        position_id: Uuid,
        margin_delta: Decimal,
    },
    DrawdownChange {
        new_drawdown: Decimal,
    },
    RecoveryTransition {
        new_state: RecoveryState,
        reason: String,
    },
}
