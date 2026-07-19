use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Sector {
    Forex,
    Indices,
    Crypto,
    Metals,
    Commodities,
    Equities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorExposure {
    pub sector: Sector,
    pub total_exposure: Decimal,
    pub concentration: Decimal,
    pub dominance: Decimal,
}

impl SectorExposure {
    pub fn new(sector: Sector) -> Self {
        Self {
            sector,
            total_exposure: Decimal::ZERO,
            concentration: Decimal::ZERO,
            dominance: Decimal::ZERO,
        }
    }

    pub fn compute_dominance(
        exposures: &mut HashMap<Sector, SectorExposure>,
        total_portfolio_exposure: Decimal,
    ) {
        if total_portfolio_exposure == Decimal::ZERO {
            return;
        }

        for exposure in exposures.values_mut() {
            exposure.dominance =
                (exposure.total_exposure / total_portfolio_exposure) * Decimal::from(100);
            exposure.concentration = exposure.dominance; // Context dependent, could be derived differently
        }
    }
}
