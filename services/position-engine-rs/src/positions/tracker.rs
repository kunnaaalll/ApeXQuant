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
    pub side: String, // "buy" or "sell"
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

    // Financials
    pub margin_used: Option<Decimal>,
    pub commission: Option<Decimal>,
    pub swap: Option<Decimal>,
    pub leverage: Option<Decimal>,

    // Performance metrics
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub max_favorable_excursion: Decimal,
    pub max_adverse_excursion: Decimal,

    // Temporal metrics
    pub opened_at: SystemTime,
    pub last_updated_at: SystemTime,
    pub closed_at: Option<SystemTime>,
}

impl PositionTracker {
    pub fn new(
        position_id: Uuid,
        symbol: String,
        side: String,
        initial_size: Decimal,
        initial_entry_price: Decimal,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            position_id,
            symbol,
            side,
            state: PositionState::Opening,
            initial_size,
            current_size: initial_size,
            initial_entry_price,
            average_entry_price: initial_entry_price,
            current_price: initial_entry_price,
            initial_stop_loss: None,
            current_stop_loss: None,
            initial_take_profit: None,
            margin_used: None,
            commission: Some(Decimal::ZERO),
            swap: Some(Decimal::ZERO),
            leverage: Some(Decimal::ONE),
            unrealized_pnl: Decimal::ZERO,
            realized_pnl: Decimal::ZERO,
            max_favorable_excursion: Decimal::ZERO,
            max_adverse_excursion: Decimal::ZERO,
            opened_at: now,
            last_updated_at: now,
            closed_at: None,
        }
    }

    pub fn update_price(&mut self, new_price: Decimal) {
        self.current_price = new_price;
        self.last_updated_at = SystemTime::now();

        // Calculate MFE and MAE
        if self.side == "buy" {
            if new_price > self.max_favorable_excursion || self.max_favorable_excursion.is_zero() {
                self.max_favorable_excursion = new_price;
            }
            if new_price < self.max_adverse_excursion || self.max_adverse_excursion.is_zero() {
                self.max_adverse_excursion = new_price;
            }
        } else {
            if new_price < self.max_favorable_excursion || self.max_favorable_excursion.is_zero() {
                self.max_favorable_excursion = new_price;
            }
            if new_price > self.max_adverse_excursion || self.max_adverse_excursion.is_zero() {
                self.max_adverse_excursion = new_price;
            }
        }
    }
}
