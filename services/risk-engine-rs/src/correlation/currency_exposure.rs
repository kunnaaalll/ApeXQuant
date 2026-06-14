//! Currency exposure analysis

use crate::correlation::pair_correlation::Currency;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Currency exposure breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyExposure {
    /// Exposure breakdown by currency
    pub breakdown: HashMap<String, Decimal>,

    /// Primary risk currency (highest exposure)
    pub primary_currency: String,

    /// Total directional exposure
    pub total_directional_exposure: Decimal,

    /// Net long exposure
    pub net_long: Decimal,

    /// Net short exposure
    pub net_short: Decimal,
}

/// Analyzer for currency exposure
pub struct CurrencyExposureAnalyzer;

impl CurrencyExposureAnalyzer {
    /// Create new analyzer
    pub fn new() -> Self {
        Self
    }

    /// Analyze currency exposure for current positions plus proposed trade
    pub fn analyze(
        &self,
        new_symbol: &str,
        positions: &[(String, Decimal, i8)],
        new_direction: i8,
    ) -> CurrencyExposure {
        let mut breakdown: HashMap<String, Decimal> = HashMap::new();
        let mut net_long = Decimal::ZERO;
        let mut net_short = Decimal::ZERO;

        // Analyze existing positions
        for (symbol, size, direction) in positions {
            let (base, quote) = Currency::from_symbol(symbol);

            // Long position: long base, short quote
            // Short position: short base, long quote
            let base_exposure = if *direction > 0 { *size } else { -*size };
            let quote_exposure = if *direction > 0 { -*size } else { *size };

            *breakdown.entry(format!("{:?}", base)).or_insert(Decimal::ZERO) += base_exposure.abs();
            *breakdown.entry(format!("{:?}", quote)).or_insert(Decimal::ZERO) += quote_exposure.abs();

            if base_exposure > Decimal::ZERO {
                net_long += base_exposure;
            } else {
                net_short += base_exposure.abs();
            }

            if quote_exposure > Decimal::ZERO {
                net_long += quote_exposure;
            } else {
                net_short += quote_exposure.abs();
            }
        }

        // Add proposed trade
        let (new_base, new_quote) = Currency::from_symbol(new_symbol);
        let new_size = Decimal::ONE; // Normalized for analysis

        let base_exposure = if new_direction > 0 { new_size } else { -new_size };
        let quote_exposure = if new_direction > 0 { -new_size } else { new_size };

        *breakdown.entry(format!("{:?}", new_base)).or_insert(Decimal::ZERO) += new_size;
        *breakdown.entry(format!("{:?}", new_quote)).or_insert(Decimal::ZERO) += new_size;

        // Find primary currency
        let primary_currency = breakdown
            .iter()
            .max_by_key(|(_, v)| v.abs())
            .map(|(k, _)| k.clone())
            .unwrap_or_else(|| "UNKNOWN".to_string());

        CurrencyExposure {
            breakdown,
            primary_currency,
            total_directional_exposure: net_long + net_short,
            net_long,
            net_short,
        }
    }

    /// Check if currency concentration is too high
    pub fn is_concentrated(&self, exposure: &CurrencyExposure, max_pct: Decimal) -> Option<String> {
        let total = exposure.net_long + exposure.net_short;

        if total == Decimal::ZERO {
            return None;
        }

        for (currency, amount) in &exposure.breakdown {
            let pct = *amount / total;
            if pct > max_pct {
                return Some(format!("{}: {:.1}%", currency, pct * Decimal::from(100)));
            }
        }

        None
    }

    /// Calculate USD exposure specifically
    pub fn usd_exposure(&self, positions: &[(String, Decimal, i8)], new_symbol: &str) -> Decimal {
        let mut usd_exposure = Decimal::ZERO;

        for (symbol, size, direction) in positions {
            let (base, quote) = Currency::from_symbol(symbol);

            // If USD is base or quote, calculate exposure
            if matches!(base, Currency::USD) {
                usd_exposure += if *direction > 0 { *size } else { -*size };
            }
            if matches!(quote, Currency::USD) {
                usd_exposure += if *direction > 0 { -*size } else { *size };
            }
        }

        // Add new symbol
        let (base, quote) = Currency::from_symbol(new_symbol);
        if matches!(base, Currency::USD) || matches!(quote, Currency::USD) {
            usd_exposure += Decimal::ONE;
        }

        usd_exposure
    }

    /// Calculate net exposure for a specific currency
    pub fn currency_net_exposure(
        &self,
        currency: Currency,
        positions: &[(String, Decimal, i8)],
    ) -> Decimal {
        let mut net = Decimal::ZERO;

        for (symbol, size, direction) in positions {
            let (base, quote) = Currency::from_symbol(symbol);

            if base == currency {
                net += if *direction > 0 { *size } else { -*size };
            }
            if quote == currency {
                net += if *direction > 0 { -*size } else { *size };
            }
        }

        net
    }

    /// Check for excessive long/short imbalance
    pub fn imbalance_ratio(&self, exposure: &CurrencyExposure) -> Option<Decimal> {
        if exposure.net_long == Decimal::ZERO && exposure.net_short == Decimal::ZERO {
            return None;
        }

        let total = exposure.net_long + exposure.net_short;
        let imbalance = (exposure.net_long - exposure.net_short).abs();

        Some(imbalance / total)
    }
}

impl Default for CurrencyExposureAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_positions() {
        let analyzer = CurrencyExposureAnalyzer::new();
        let exposure = analyzer.analyze("EURUSD", &[], 1);

        assert!(exposure.breakdown.contains_key("EUR"));
        assert!(exposure.breakdown.contains_key("USD"));
    }

    #[test]
    fn test_currency_net_exposure() {
        let analyzer = CurrencyExposureAnalyzer::new();
        let positions = vec![
            ("EURUSD".to_string(), Decimal::from(1), 1),
        ];

        let eur_exposure = analyzer.currency_net_exposure(Currency::EUR, &positions);
        assert_eq!(eur_exposure, Decimal::from(1));

        let usd_exposure = analyzer.currency_net_exposure(Currency::USD, &positions);
        assert_eq!(usd_exposure, Decimal::from(-1));
    }

    #[test]
    fn test_usd_exposure() {
        let analyzer = CurrencyExposureAnalyzer::new();
        let positions = vec![
            ("EURUSD".to_string(), Decimal::from(1), 1),
            ("USDJPY".to_string(), Decimal::from(1), 1),
        ];

        let usd = analyzer.usd_exposure(&positions, "GBPUSD");
        // EURUSD short USD: -1, USDJPY long USD: +1, GBPUSD short USD: -1
        assert_eq!(usd, Decimal::from(-1));
    }
}
