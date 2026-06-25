use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LearningEvent {
    TradeCompleted {
        trade_id: Uuid,
        strategy_id: Uuid,
        symbol: String,
        pnl: Decimal,
        timestamp: OffsetDateTime,
    },
    MarketConditionShifted {
        regime_id: Uuid,
        volatility: Decimal,
        trend: Decimal,
        timestamp: OffsetDateTime,
    },
    StrategyStateChanged {
        strategy_id: Uuid,
        is_active: bool,
        allocation: Decimal,
        timestamp: OffsetDateTime,
    },
    RiskAlertTriggered {
        alert_id: Uuid,
        severity: String,
        timestamp: OffsetDateTime,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearningSnapshot {
    pub sequence_number: u64,
    pub active_strategies: std::collections::HashMap<Uuid, StrategyStateSnapshot>,
    pub timestamp: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyStateSnapshot {
    pub allocation: Decimal,
    pub is_active: bool,
    pub total_pnl: Decimal,
    pub trade_count: u64,
}

pub struct LearningRebuilder {
    pub state: LearningSnapshot,
}

impl LearningRebuilder {
    pub fn new(initial_state: Option<LearningSnapshot>) -> Self {
        Self {
            state: initial_state.unwrap_or_default(),
        }
    }

    pub fn apply_event(&mut self, event: &LearningEvent, sequence_number: u64) {
        match event {
            LearningEvent::TradeCompleted {
                strategy_id,
                pnl,
                timestamp,
                ..
            } => {
                let strategy = self.state.active_strategies.entry(*strategy_id).or_insert(
                    StrategyStateSnapshot {
                        allocation: Decimal::ZERO,
                        is_active: true,
                        total_pnl: Decimal::ZERO,
                        trade_count: 0,
                    },
                );
                strategy.total_pnl += pnl;
                strategy.trade_count += 1;
                self.state.timestamp = Some(*timestamp);
            }
            LearningEvent::StrategyStateChanged {
                strategy_id,
                is_active,
                allocation,
                timestamp,
            } => {
                let strategy = self.state.active_strategies.entry(*strategy_id).or_insert(
                    StrategyStateSnapshot {
                        allocation: Decimal::ZERO,
                        is_active: true,
                        total_pnl: Decimal::ZERO,
                        trade_count: 0,
                    },
                );
                strategy.is_active = *is_active;
                strategy.allocation = *allocation;
                self.state.timestamp = Some(*timestamp);
            }
            LearningEvent::MarketConditionShifted { timestamp, .. } => {
                self.state.timestamp = Some(*timestamp);
            }
            LearningEvent::RiskAlertTriggered { timestamp, .. } => {
                self.state.timestamp = Some(*timestamp);
            }
        }
        self.state.sequence_number = sequence_number;
    }

    pub fn apply_events(&mut self, events: &[(u64, LearningEvent)]) {
        for (seq_num, event) in events {
            self.apply_event(event, *seq_num);
        }
    }
}
