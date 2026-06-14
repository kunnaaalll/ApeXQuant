//! Pair-wise correlation calculations
use rust_decimal::prelude::FromPrimitive;

use rust_decimal::Decimal;
use std::collections::HashMap;

/// Currency pair classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CHF,
    AUD,
    CAD,
    NZD,
    Other,
}

impl Currency {
    /// Parse currency from symbol
    pub fn from_symbol(symbol: &str) -> (Self, Self) {
        if symbol.len() >= 6 {
            let base = &symbol[0..3];
            let quote = &symbol[3..6];
            (Self::from_str(base), Self::from_str(quote))
        } else {
            (Self::Other, Self::Other)
        }
    }

    fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "USD" => Self::USD,
            "EUR" => Self::EUR,
            "GBP" => Self::GBP,
            "JPY" => Self::JPY,
            "CHF" => Self::CHF,
            "AUD" => Self::AUD,
            "CAD" => Self::CAD,
            "NZD" => Self::NZD,
            _ => Self::Other,
        }
    }
}

/// Correlation level between currency pairs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CorrelationLevel {
    /// Strong positive correlation (> 0.8)
    StrongPositive,
    /// Positive correlation (0.5 - 0.8)
    Positive,
    /// Weak correlation (-0.5 to 0.5)
    Weak,
    /// Negative correlation (-0.5 to -0.8)
    Negative,
    /// Strong negative correlation (< -0.8)
    StrongNegative,
}

impl CorrelationLevel {
    /// Get the correlation value
    pub fn value(&self) -> Decimal {
        match self {
            CorrelationLevel::StrongPositive => Decimal::from_f64(0.9).unwrap(),
            CorrelationLevel::Positive => Decimal::from_f64(0.65).unwrap(),
            CorrelationLevel::Weak => Decimal::ZERO,
            CorrelationLevel::Negative => Decimal::from_f64(-0.65).unwrap(),
            CorrelationLevel::StrongNegative => Decimal::from_f64(-0.9).unwrap(),
        }
    }
}

/// Engine for calculating pair correlations
#[derive(Debug, Clone, Default)]
pub struct PairCorrelationEngine {
    /// Cached correlation matrix
    correlations: HashMap<String, Decimal>,
}

impl PairCorrelationEngine {
    /// Get correlation between two symbols
    ///
    /// Returns a value between -1 and 1 where:
    /// - 1 = perfectly correlated (move together)
    /// - -1 = perfectly inversely correlated
    /// - 0 = uncorrelated
    pub fn correlation(&self, symbol1: &str, symbol2: &str) -> Decimal {
        if symbol1 == symbol2 {
            return Decimal::ONE;
        }

        // Check cache
        let key = Self::make_key(symbol1, symbol2);
        if let Some(&corr) = self.correlations.get(&key) {
            return corr;
        }

        // Calculate based on currency relationships
        let corr = self.calculate_correlation(symbol1, symbol2);

        // Store in cache (would need mutability in real impl)
        // self.correlations.insert(key, corr);

        corr
    }

    fn calculate_correlation(&self, symbol1: &str, symbol2: &str) -> Decimal {
        let (base1, quote1) = Currency::from_symbol(symbol1);
        let (base2, quote2) = Currency::from_symbol(symbol2);

        // Same pair - perfect correlation
        if symbol1 == symbol2 {
            return Decimal::ONE;
        }

        // Inverse pair (EURUSD vs USDCHF is negative correlation)
        if base1 == quote2 && quote1 == base2 {
            return Decimal::from_f64(-0.95).unwrap();
        }

        // Same base currency (EURUSD vs EURGBP - correlated)
        if base1 == base2 {
            return Decimal::from_f64(0.85).unwrap();
        }

        // Same quote currency (EURUSD vs GBPUSD - correlated)
        if quote1 == quote2 {
            return Decimal::from_f64(0.80).unwrap();
        }

        // USD cross pairs (EURUSD vs USDJPY - inversely correlated via USD)
        if (quote1 == Currency::USD && base2 == Currency::USD)
            || (base1 == Currency::USD && quote2 == Currency::USD)
        {
            return Decimal::from_f64(-0.70).unwrap();
        }

        // Commodity currencies (AUD, NZD, CAD correlate with commodities)
        if Self::both_commodity(base1, base2) {
            return Decimal::from_f64(0.75).unwrap();
        }

        // Safe havens (JPY, CHF correlate negatively with risk)
        if Self::both_safe_haven(base1, base2) {
            return Decimal::from_f64(0.70).unwrap();
        }

        // Default - weak correlation
        Decimal::from_f64(0.2).unwrap()
    }

    fn make_key(a: &str, b: &str) -> String {
        if a < b {
            format!("{}:{}", a, b)
        } else {
            format!("{}:{}", b, a)
        }
    }

    fn both_commodity(c1: Currency, c2: Currency) -> bool {
        matches!((c1, c2), (Currency::AUD, Currency::NZD)
            | (Currency::AUD, Currency::CAD)
            | (Currency::NZD, Currency::CAD)
            | (Currency::NZD, Currency::AUD)
            | (Currency::CAD, Currency::AUD)
            | (Currency::CAD, Currency::NZD))
    }

    fn both_safe_haven(c1: Currency, c2: Currency) -> bool {
        matches!((c1, c2), (Currency::JPY, Currency::CHF) | (Currency::CHF, Currency::JPY))
    }

    /// Get correlation level description
    pub fn correlation_level(&self, correlation: Decimal) -> CorrelationLevel {
        if correlation >= Decimal::from_f64(0.8).unwrap() {
            CorrelationLevel::StrongPositive
        } else if correlation >= Decimal::from_f64(0.5).unwrap() {
            CorrelationLevel::Positive
        } else if correlation <= Decimal::from_f64(-0.8).unwrap() {
            CorrelationLevel::StrongNegative
        } else if correlation <= Decimal::from_f64(-0.5).unwrap() {
            CorrelationLevel::Negative
        } else {
            CorrelationLevel::Weak
        }
    }

    /// Calculate portfolio correlation risk
    pub fn portfolio_correlation(
        &self,
        new_symbol: &str,
        positions: &[(String, Decimal, i8)],
    ) -> Decimal {
        if positions.is_empty() {
            return Decimal::ZERO;
        }

        let weighted_sum: Decimal = positions
            .iter()
            .map(|(sym, size, dir)| {
                let corr = self.correlation(new_symbol, sym);
                // Direction matters - same direction with positive correlation increases risk
                let adjusted_corr = if *dir > 0 { corr } else { -corr };
                adjusted_corr * size.abs()
            })
            .sum();

        let total_size: Decimal = positions.iter().map(|(_, size, _)| size.abs()).sum();

        if total_size > Decimal::ZERO {
            weighted_sum / total_size
        } else {
            Decimal::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_symbol_correlation() {
        let engine = PairCorrelationEngine::default();
        assert_eq!(
            engine.correlation("EURUSD", "EURUSD"),
            Decimal::ONE
        );
    }

    #[test]
    fn test_inverse_pair_correlation() {
        let engine = PairCorrelationEngine::default();
        let corr = engine.correlation("EURUSD", "USDCHF");
        assert!(corr < Decimal::ZERO);
    }

    #[test]
    fn test_same_base_correlation() {
        let engine = PairCorrelationEngine::default();
        let corr = engine.correlation("EURUSD", "EURGBP");
        assert!(corr > Decimal::from_f64(0.5).unwrap());
    }

    #[test]
    fn test_currency_parsing() {
        let (base, quote) = Currency::from_symbol("EURUSD");
        assert!(matches!(base, Currency::EUR));
        assert!(matches!(quote, Currency::USD));
    }

    #[test]
    fn test_correlation_level() {
        let engine = PairCorrelationEngine::default();
        assert_eq!(
            engine.correlation_level(Decimal::from_f64(0.9).unwrap()),
            CorrelationLevel::StrongPositive
        );
        assert_eq!(
            engine.correlation_level(Decimal::from_f64(-0.9).unwrap()),
            CorrelationLevel::StrongNegative
        );
    }
}
