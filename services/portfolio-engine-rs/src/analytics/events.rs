// src/analytics/events.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalyticsEventType {
    PositionClosed {
        symbol: String,
        pnl: f64,
        duration_ms: u64,
    },
    PnLUpdate {
        realized_pnl: f64,
        unrealized_pnl: f64,
    },
    DrawdownUpdate {
        current_drawdown: f64,
        max_drawdown: f64,
    },
    PortfolioUpdate {
        total_value: f64,
        cash_balance: f64,
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
