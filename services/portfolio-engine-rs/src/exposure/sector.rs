use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Sector {
    Forex,
    Indices,
    Metals,
    Crypto,
    Commodities,
    Bonds,
    Synthetic,
    Other(uuid::Uuid),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SectorExposure {
    pub sector: Sector,
    pub capital_allocated: Decimal,
    pub weight: Decimal,
    pub pnl_contribution: Decimal,
    pub risk_contribution: Decimal,
    pub position_count: usize,
}

impl SectorExposure {
    pub fn new(sector: Sector) -> Self {
        Self {
            sector,
            capital_allocated: Decimal::ZERO,
            weight: Decimal::ZERO,
            pnl_contribution: Decimal::ZERO,
            risk_contribution: Decimal::ZERO,
            position_count: 0,
        }
    }
}
