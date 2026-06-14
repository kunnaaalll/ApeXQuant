use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HeatEvent {
    PositionOpened {
        symbol_id: String,
        risk_amount: Decimal,
        margin_used: Decimal,
    },
    PositionClosed {
        symbol_id: String,
        risk_released: Decimal,
        margin_released: Decimal,
    },
    ScaleIn {
        symbol_id: String,
        additional_risk: Decimal,
        additional_margin: Decimal,
    },
    ScaleOut {
        symbol_id: String,
        risk_released: Decimal,
        margin_released: Decimal,
    },
    PnlChanged {
        symbol_id: String,
        pnl_delta: Decimal,
        new_total_pnl: Decimal,
    },
    DrawdownChanged {
        drawdown_percentage: Decimal,
        is_new_high: bool,
    },
    CircuitBreakerActivated {
        reason: String,
        duration_seconds: u64,
    },
    RecoveryTransition {
        from_state: String,
        to_state: String,
    },
    VolatilityChanged {
        new_regime: String,
        volatility_index: Decimal,
    },
    HeatDecayTick {
        decay_amount: Decimal,
        new_heat: u8,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeatSnapshot {
    pub timestamp: OffsetDateTime,
    pub heat_score: u8,
    pub state: super::heat_score::PortfolioHeatState,
    pub version: u64,
    pub breakdown: super::heat_score::HeatContributionBreakdown,
    pub trigger_event: HeatEvent,
}

impl HeatSnapshot {
    pub fn new(
        timestamp: OffsetDateTime,
        heat_score: u8,
        state: super::heat_score::PortfolioHeatState,
        version: u64,
        breakdown: super::heat_score::HeatContributionBreakdown,
        trigger_event: HeatEvent,
    ) -> Self {
        Self {
            timestamp,
            heat_score,
            state,
            version,
            breakdown,
            trigger_event,
        }
    }
}
