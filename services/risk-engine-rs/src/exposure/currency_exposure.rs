use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyExposure {
    pub currency: String,
    pub gross_exposure: Decimal,
    pub net_exposure: Decimal,
    pub concentration_ratio: Decimal,
}

impl CurrencyExposure {
    pub fn new(currency: String) -> Self {
        Self {
            currency,
            gross_exposure: Decimal::ZERO,
            net_exposure: Decimal::ZERO,
            concentration_ratio: Decimal::ZERO,
        }
    }

    pub fn is_short_usd_cluster(exposures: &HashMap<String, CurrencyExposure>, threshold: Decimal) -> bool {
        if let Some(usd) = exposures.get("USD") {
            if usd.net_exposure < Decimal::ZERO && usd.net_exposure.abs() > threshold {
                return true;
            }
        }
        false
    }

    pub fn check_excessive_dependency(&self, max_allowed_ratio: Decimal) -> bool {
        self.concentration_ratio > max_allowed_ratio
    }
}

pub fn decompose_synthetic(
    pair: &str,
    is_long: bool,
    amount: Decimal,
) -> Result<(String, Decimal, String, Decimal), &'static str> {
    if pair.len() != 6 {
        return Err("Invalid pair length");
    }

    let base = pair[0..3].to_string();
    let quote = pair[3..6].to_string();

    if is_long {
        Ok((base, amount, quote, -amount))
    } else {
        Ok((base, -amount, quote, amount))
    }
}
