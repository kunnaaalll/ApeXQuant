//! Trade Consumer — decodes completed trade events from the Event Bus.

use rust_decimal::prelude::FromStr;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Normalised completed trade record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedTrade {
    pub trade_id: String,
    pub account_id: String,
    pub symbol: String,
    pub strategy_id: String,
    pub is_long: bool,
    pub entry_price: Decimal,
    pub exit_price: Decimal,
    pub quantity: Decimal,
    pub commission: Decimal,
    pub swap: Decimal,
    pub gross_pnl: Decimal,
    pub net_pnl: Decimal,
    pub entry_time_ms: i64,
    pub exit_time_ms: i64,
    pub mae: Decimal,
    pub mfe: Decimal,
    pub r_multiple: Decimal,
}

impl CompletedTrade {
    /// Holding duration in seconds.
    pub fn holding_duration_secs(&self) -> i64 {
        (self.exit_time_ms - self.entry_time_ms) / 1000
    }

    /// Is the trade a winner?
    pub fn is_winner(&self) -> bool {
        self.net_pnl > Decimal::ZERO
    }
}

/// Parse a completed trade from a raw JSON event payload.
pub fn parse_trade_event(payload: &[u8]) -> Result<CompletedTrade, String> {
    serde_json::from_slice::<serde_json::Value>(payload)
        .map_err(|e| format!("JSON parse error: {}", e))
        .and_then(|v| parse_trade_from_value(&v))
}

fn parse_trade_from_value(v: &serde_json::Value) -> Result<CompletedTrade, String> {
    let get_str = |field: &str| -> Result<String, String> {
        v[field]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| format!("Missing or invalid field: {}", field))
    };
    let get_decimal = |field: &str| -> Result<Decimal, String> {
        v[field]
            .as_str()
            .ok_or_else(|| format!("Missing decimal field: {}", field))
            .and_then(|s| {
                Decimal::from_str(s).map_err(|e| format!("Invalid decimal {}: {}", field, e))
            })
    };
    let get_i64 = |field: &str| -> Result<i64, String> {
        v[field]
            .as_i64()
            .ok_or_else(|| format!("Missing or invalid i64: {}", field))
    };
    let get_bool = |field: &str| -> Result<bool, String> {
        v[field]
            .as_bool()
            .ok_or_else(|| format!("Missing or invalid bool: {}", field))
    };

    Ok(CompletedTrade {
        trade_id: get_str("trade_id")?,
        account_id: get_str("account_id")?,
        symbol: get_str("symbol")?,
        strategy_id: get_str("strategy_id")?,
        is_long: get_bool("is_long")?,
        entry_price: get_decimal("entry_price")?,
        exit_price: get_decimal("exit_price")?,
        quantity: get_decimal("quantity")?,
        commission: get_decimal("commission").unwrap_or(Decimal::ZERO),
        swap: get_decimal("swap").unwrap_or(Decimal::ZERO),
        gross_pnl: get_decimal("gross_pnl")?,
        net_pnl: get_decimal("net_pnl")?,
        entry_time_ms: get_i64("entry_time_ms")?,
        exit_time_ms: get_i64("exit_time_ms")?,
        mae: get_decimal("mae").unwrap_or(Decimal::ZERO),
        mfe: get_decimal("mfe").unwrap_or(Decimal::ZERO),
        r_multiple: get_decimal("r_multiple").unwrap_or(Decimal::ZERO),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_trade() {
        let json = serde_json::json!({
            "trade_id": "t-001",
            "account_id": "acc-1",
            "symbol": "EURUSD",
            "strategy_id": "strat-1",
            "is_long": true,
            "entry_price": "1.10000",
            "exit_price": "1.10500",
            "quantity": "10000",
            "commission": "5.00",
            "swap": "0.00",
            "gross_pnl": "50.00",
            "net_pnl": "45.00",
            "entry_time_ms": 1000000,
            "exit_time_ms": 1003600000,
            "mae": "10.00",
            "mfe": "60.00",
            "r_multiple": "1.5"
        });
        let bytes = serde_json::to_vec(&json).expect("json encode");
        let trade = parse_trade_event(&bytes).expect("parse failed");
        assert_eq!(trade.trade_id, "t-001");
        assert_eq!(trade.net_pnl, Decimal::new(45, 0));
        assert!(trade.is_winner());
    }
}
