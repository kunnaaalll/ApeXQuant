use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

use super::state::PositionState;

/// The core tracking entity for a single position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionTracker {
    pub position_id: Uuid,
    pub symbol: String,
    pub state: PositionState,

    // Core sizing metrics
    pub initial_size: Decimal,
    pub current_size: Decimal,

    // Core pricing metrics
    pub initial_entry_price: Decimal,
    pub average_entry_price: Decimal,
    pub current_price: Decimal,

    // Risk configuration
    pub initial_stop_loss: Option<Decimal>,
    pub current_stop_loss: Option<Decimal>,
    pub initial_take_profit: Option<Decimal>,

    // Performance metrics
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub max_favorable_excursion: Decimal,
    pub max_adverse_excursion: Decimal,

    // Temporal metrics
    pub opened_at: SystemTime,
    pub last_updated_at: SystemTime,
}

impl PositionTracker {
    pub fn new(
        position_id: Uuid,
        symbol: String,
        initial_size: Decimal,
        initial_entry_price: Decimal,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            position_id,
            symbol,
            state: PositionState::Opening,
            initial_size,
            current_size: initial_size,
            initial_entry_price,
            average_entry_price: initial_entry_price,
            current_price: initial_entry_price,
            initial_stop_loss: None,
            current_stop_loss: None,
            initial_take_profit: None,
            unrealized_pnl: Decimal::ZERO,
            realized_pnl: Decimal::ZERO,
            max_favorable_excursion: Decimal::ZERO,
            max_adverse_excursion: Decimal::ZERO,
            opened_at: now,
            last_updated_at: now,
        }
    }

    pub fn update_price(&mut self, new_price: Decimal) {
        self.current_price = new_price;
        self.last_updated_at = SystemTime::now();

        // TODO: Update MFE/MAE and Unrealized PnL internally or via event
    }
}
