use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub type TradeCount = u32;
pub type Ratio = Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeframeType {
    M15,
    M30,
    H1,
    H4,
}
