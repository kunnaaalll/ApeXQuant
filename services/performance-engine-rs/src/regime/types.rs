use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub type TradeCount = u32;
pub type Ratio = Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegimeType {
    Trending,
    Ranging,
    Expansion,
    Contraction,
    HighVolatility,
    LowVolatility,
}
