// src/analytics/events.rs
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalyticsEventType {
    PositionClosed {
        symbol: String,
        pnl: Decimal,
        duration_ms: u64,
    },
    PnLUpdate {
        realized_pnl: Decimal,
        unrealized_pnl: Decimal,
    },
    DrawdownUpdate {
        current_drawdown: Decimal,
        max_drawdown: Decimal,
    },
    PortfolioUpdate {
        total_value: Decimal,
        cash_balance: Decimal,
    },
}

/// An event that triggers an analytics recalculation.
/// APEX enforces no silent mutations, so any state change
/// must be driven by an explicit event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyticsEvent {
    pub event_id: String,
    pub timestamp: i64,
    pub event_type: AnalyticsEventType,
}

impl AnalyticsEvent {
    pub fn new(event_id: String, timestamp: i64, event_type: AnalyticsEventType) -> Self {
        Self {
            event_id,
            timestamp,
            event_type,
        }
    }
}
