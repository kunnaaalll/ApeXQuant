use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub type TradeCount = u32;
pub type Ratio = Decimal;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol(pub String);
