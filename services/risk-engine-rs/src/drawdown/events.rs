use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DrawdownEvent {
    Started { timestamp: i64, value: Decimal },
    Updated { timestamp: i64, value: Decimal },
    Recovered { timestamp: i64 },
}
