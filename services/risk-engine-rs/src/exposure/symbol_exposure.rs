use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolExposure {
    pub symbol: String,
    pub weight: Decimal,
    pub percentage_of_capital: Decimal,
    pub long_exposure: Decimal,
    pub short_exposure: Decimal,
}

impl SymbolExposure {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            weight: Decimal::ZERO,
            percentage_of_capital: Decimal::ZERO,
            long_exposure: Decimal::ZERO,
            short_exposure: Decimal::ZERO,
        }
    }

    pub fn net_exposure(&self) -> Decimal {
        self.long_exposure - self.short_exposure
    }

    pub fn is_oversized(&self, threshold: Decimal) -> bool {
        self.percentage_of_capital > threshold
    }

    pub fn detect_concentration_breach(&self, limit: Decimal) -> bool {
        self.weight > limit
    }
}
