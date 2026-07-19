use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::drawdown_state::DrawdownState;
use super::metrics::{RecoveryFactor, TimeUnderWaterAssessment, UlcerIndexAssessment};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DrawdownEvent {
    PnlChanged {
        timestamp: time::OffsetDateTime,
        amount: Decimal,
    },
    PositionOpened {
        timestamp: time::OffsetDateTime,
        symbol: String,
    },
    PositionClosed {
        timestamp: time::OffsetDateTime,
        symbol: String,
        realized_pnl: Decimal,
    },
    StateTransitioned {
        timestamp: time::OffsetDateTime,
        from: DrawdownState,
        to: DrawdownState,
    },
    RecoveryProgressed {
        timestamp: time::OffsetDateTime,
        recovery_score: Decimal,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotTimeframe {
    Realtime,
    M1,
    M5,
    M15,
    H1,
    D1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownSnapshot {
    pub version: u64,
    pub timestamp: time::OffsetDateTime,
    pub timeframe: SnapshotTimeframe,

    pub state: DrawdownState,

    pub daily_drawdown: Decimal,
    pub weekly_drawdown: Decimal,
    pub monthly_drawdown: Decimal,
    pub rolling_drawdown: Decimal,
    pub peak_to_valley: Decimal,

    pub equity_efficiency: Decimal,

    pub time_under_water: TimeUnderWaterAssessment,
    pub recovery_factor: RecoveryFactor,
    pub ulcer_index: UlcerIndexAssessment,
}
