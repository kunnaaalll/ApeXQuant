use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CHF,
    CAD,
    AUD,
    NZD,
    XAU,
    BTC,
    Other(uuid::Uuid), // Fallback
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurrencyExposure {
    pub currency: Currency,
    pub net_exposure: Decimal,
    pub gross_exposure: Decimal,
    pub long_exposure: Decimal,
    pub short_exposure: Decimal,
    pub percentage_contribution: Decimal,
}

impl CurrencyExposure {
    pub fn new(currency: Currency) -> Self {
        Self {
            currency,
            net_exposure: Decimal::ZERO,
            gross_exposure: Decimal::ZERO,
            long_exposure: Decimal::ZERO,
            short_exposure: Decimal::ZERO,
            percentage_contribution: Decimal::ZERO,
        }
    }
}
