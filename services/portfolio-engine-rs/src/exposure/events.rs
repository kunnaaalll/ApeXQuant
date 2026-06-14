use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::currency::Currency;
use super::sector::Sector;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExposureEvent {
    PositionOpened {
        position_id: Uuid,
        symbol_id: String,
        sector: Sector,
        base_currency: Currency,
        quote_currency: Currency,
        base_size: Decimal, // Positive for Long, Negative for Short
        quote_size: Decimal, // Opposite sign of base_size based on entry price
        margin_used: Decimal,
        risk_amount: Decimal,
    },
    PositionClosed {
        position_id: Uuid,
        symbol_id: String,
        sector: Sector,
        base_currency: Currency,
        quote_currency: Currency,
        base_size_released: Decimal,
        quote_size_released: Decimal,
        margin_released: Decimal,
        risk_released: Decimal,
    },
    ScaleIn {
        position_id: Uuid,
        symbol_id: String,
        base_size_added: Decimal,
        quote_size_added: Decimal,
        margin_added: Decimal,
        risk_added: Decimal,
    },
    ScaleOut {
        position_id: Uuid,
        symbol_id: String,
        base_size_released: Decimal,
        quote_size_released: Decimal,
        margin_released: Decimal,
        risk_released: Decimal,
    },
    PartialClose {
        position_id: Uuid,
        symbol_id: String,
        base_size_released: Decimal,
        quote_size_released: Decimal,
        margin_released: Decimal,
        risk_released: Decimal,
    },
    SymbolAdded {
        symbol_id: String,
        sector: Sector,
    },
    SymbolRemoved {
        symbol_id: String,
    },
    PnlChanged {
        position_id: Uuid,
        symbol_id: String,
        pnl_delta: Decimal,
    },
}
