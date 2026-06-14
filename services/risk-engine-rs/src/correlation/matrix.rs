//! Correlation matrix for portfolio analysis
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::MathematicalOps;

use crate::correlation::pair_correlation::Currency;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Correlation matrix for tracking symbol correlations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    /// Matrix of correlations
    values: HashMap<(String, String), Decimal>,
    /// Symbols in matrix
    symbols: Vec<String>,
}

impl CorrelationMatrix {
    /// Create empty correlation matrix
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            symbols: Vec::new(),
        }
    }

    /// Create matrix with symbols
    pub fn with_symbols(symbols: Vec<String>) -> Self {
        Self {
            values: HashMap::new(),
            symbols,
        }
    }

    /// Get correlation between two symbols
    pub fn get(&self, symbol1: &str, symbol2: &str) -> Option<Decimal> {
        // Try both orderings
        self.values
            .get(&(symbol1.to_string(), symbol2.to_string()))
            .or_else(|| self.values.get(&(symbol2.to_string(), symbol1.to_string())))
            .copied()
    }

    /// Set correlation between two symbols
    pub fn set(&mut self, symbol1: &str, symbol2: &str, correlation: Decimal) {
        self.values.insert(
            (symbol1.to_string(), symbol2.to_string()),
            correlation.clamp(Decimal::NEGATIVE_ONE, Decimal::ONE),
        );

        // Add symbols if not present
        if !self.symbols.contains(&symbol1.to_string()) {
            self.symbols.push(symbol1.to_string());
        }
        if !self.symbols.contains(&symbol2.to_string()) {
            self.symbols.push(symbol2.to_string());
        }
    }

    /// Add a symbol and calculate correlations with existing symbols
    pub fn add_symbol(&mut self, symbol: &str, correlations: &[(String, Decimal)]) {
        for (other, corr) in correlations {
            if self.symbols.contains(other) {
                self.set(symbol, other, *corr);
            }
        }

        if !self.symbols.contains(&symbol.to_string()) {
            self.symbols.push(symbol.to_string());
        }
    }

    /// Remove a symbol from the matrix
    pub fn remove_symbol(&mut self, symbol: &str) {
        self.symbols.retain(|s| s != symbol);

        // Remove all correlations involving this symbol
        self.values
            .retain(|(s1, s2), _| s1 != symbol && s2 != symbol);
    }

    /// Get all correlations for a symbol
    pub fn get_correlations(&self, symbol: &str) -> Vec<(&str, Decimal)> {
        self.values
            .iter()
            .filter_map(|((s1, s2), corr)| {
                if s1 == symbol {
                    Some((s2.as_str(), *corr))
                } else if s2 == symbol {
                    Some((s1.as_str(), *corr))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get highest correlated symbol
    pub fn highest_correlation(&self, symbol: &str) -> Option<(&str, Decimal)> {
        self.get_correlations(symbol)
            .into_iter()
            .max_by_key(|(_, corr)| corr.abs())
    }

    /// Calculate average correlation for a symbol
    pub fn average_correlation(&self, symbol: &str) -> Decimal {
        let correlations = self.get_correlations(symbol);

        if correlations.is_empty() {
            return Decimal::ZERO;
        }

        let sum: Decimal = correlations.iter().map(|(_, c)| c.abs()).sum();
        sum / Decimal::from(correlations.len() as u32)
    }

    /// Get portfolio correlation risk
    pub fn portfolio_risk(&self, positions: &[(String, Decimal, i8)]) -> Decimal {
        if positions.len() < 2 {
            return Decimal::ZERO;
        }

        let mut total_weighted_corr = Decimal::ZERO;
        let mut total_weight = Decimal::ZERO;

        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                let (sym1, size1, dir1) = &positions[i];
                let (sym2, size2, dir2) = &positions[j];

                let corr = self.get(sym1, sym2).unwrap_or(Decimal::ZERO);

                // Same direction amplifies risk, opposite reduces
                let direction_factor = if *dir1 == *dir2 { Decimal::ONE } else { Decimal::NEGATIVE_ONE };
                let weighted_corr = corr * direction_factor * size1.abs() * size2.abs();

                total_weighted_corr += weighted_corr;
                total_weight += size1.abs() * size2.abs();
            }
        }

        if total_weight > Decimal::ZERO {
            total_weighted_corr / total_weight
        } else {
            Decimal::ZERO
        }
    }

    /// Check if matrix contains a symbol
    pub fn contains(&self, symbol: &str) -> bool {
        self.symbols.contains(&symbol.to_string())
    }

    /// Get all symbols in matrix
    pub fn symbols(&self) -> &[String] {
        &self.symbols
    }

    /// Get matrix size
    pub fn size(&self) -> usize {
        self.symbols.len()
    }

    /// Build correlation matrix from currency relationships
    pub fn build_from_currencies(&mut self, symbols: &[String]) {
        for (i, sym1) in symbols.iter().enumerate() {
            for sym2 in symbols.iter().skip(i + 1) {
                let corr = self.calculate_currency_correlation(sym1, sym2);
                self.set(sym1, sym2, corr);
            }
        }
    }

    fn calculate_currency_correlation(&self, symbol1: &str, symbol2: &str) -> Decimal {
        let (base1, quote1) = Currency::from_symbol(symbol1);
        let (base2, quote2) = Currency::from_symbol(symbol2);

        // Same symbol
        if symbol1 == symbol2 {
            return Decimal::ONE;
        }

        // Inverse relationship
        if base1 == quote2 && quote1 == base2 {
            return Decimal::from_f64(-0.95).unwrap();
        }

        // Share base (e.g., EURUSD vs EURGBP)
        if base1 == base2 {
            return Decimal::from_f64(0.85).unwrap();
        }

        // Share quote (e.g., EURUSD vs GBPUSD)
        if quote1 == quote2 {
            return Decimal::from_f64(0.80).unwrap();
        }

        // Cross via major
        if (quote1 == Currency::USD && base2 == Currency::USD)
            || (base1 == Currency::USD && quote2 == Currency::USD)
        {
            return Decimal::from_f64(-0.70).unwrap();
        }

        // Commodity currencies
        if Self::is_commodity_currency(base1) && Self::is_commodity_currency(base2) {
            return Decimal::from_f64(0.70).unwrap();
        }

        // Safe havens
        if Self::is_safe_haven(base1) && Self::is_safe_haven(base2) {
            return Decimal::from_f64(0.60).unwrap();
        }

        // Emerging markets
        if Self::is_emerging(base1) && Self::is_emerging(base2) {
            return Decimal::from_f64(0.65).unwrap();
        }

        Decimal::from_f64(0.2).unwrap()
    }

    fn is_commodity_currency(curr: Currency) -> bool {
        matches!(curr, Currency::AUD | Currency::NZD | Currency::CAD)
    }

    fn is_safe_haven(curr: Currency) -> bool {
        matches!(curr, Currency::JPY | Currency::CHF)
    }

    fn is_emerging(curr: Currency) -> bool {
        matches!(curr, Currency::Other)
    }

    /// Create sector-based correlation matrix
    pub fn sector_matrix(&self, symbols: &[String]) -> HashMap<(String, String), Decimal> {
        let mut sector_corrs = HashMap::new();

        for (i, sym1) in symbols.iter().enumerate() {
            for sym2 in symbols.iter().skip(i + 1) {
                let sectors1 = crate::correlation::Sector::for_symbol(sym1);
                let sectors2 = crate::correlation::Sector::for_symbol(sym2);

                // Correlation is higher if sectors overlap
                let overlap: Vec<_> = sectors1
                    .iter()
                    .filter(|s| sectors2.contains(s))
                    .collect();

                let corr = if overlap.is_empty() {
                    Decimal::from_f64(0.2).unwrap()
                } else {
                    Decimal::from_f64(0.5 + (overlap.len() as f64 * 0.15)).unwrap()
                        .min(Decimal::ONE)
                };

                sector_corrs.insert((sym1.clone(), sym2.clone()), corr);
            }
        }

        sector_corrs
    }
}

impl Default for CorrelationMatrix {
    fn default() -> Self {
        Self::new()
    }
}

/// Rolling correlation window for dynamic updates
#[derive(Debug, Clone)]
pub struct RollingCorrelation {
    /// Window size
    window: usize,
    /// Price history per symbol
    history: HashMap<String, Vec<Decimal>>,
}

impl RollingCorrelation {
    /// Create new rolling correlation calculator
    pub fn new(window: usize) -> Self {
        Self {
            window,
            history: HashMap::new(),
        }
    }

    /// Add price observation
    pub fn add_observation(&mut self, symbol: &str, price: Decimal) {
        let entry = self.history.entry(symbol.to_string()).or_default();
        entry.push(price);

        // Maintain window size
        if entry.len() > self.window {
            entry.remove(0);
        }
    }

    /// Calculate rolling correlation between two symbols
    pub fn calculate(&self, symbol1: &str, symbol2: &str) -> Option<Decimal> {
        let prices1 = self.history.get(symbol1)?;
        let prices2 = self.history.get(symbol2)?;

        if prices1.len() < 2 || prices1.len() != prices2.len() {
            return None;
        }

        Some(Self::pearson_correlation(prices1, prices2))
    }

    fn pearson_correlation(x: &[Decimal], y: &[Decimal]) -> Decimal {
        let n = Decimal::from(x.len() as u32);

        let sum_x: Decimal = x.iter().sum();
        let sum_y: Decimal = y.iter().sum();

        let mean_x = sum_x / n;
        let mean_y = sum_y / n;

        let mut numerator = Decimal::ZERO;
        let mut sum_sq_x = Decimal::ZERO;
        let mut sum_sq_y = Decimal::ZERO;

        for i in 0..x.len() {
            let dx = x[i] - mean_x;
            let dy = y[i] - mean_y;

            numerator += dx * dy;
            sum_sq_x += dx * dx;
            sum_sq_y += dy * dy;
        }

        let denominator = (sum_sq_x * sum_sq_y).sqrt().unwrap_or(Decimal::ZERO);

        if denominator > Decimal::ZERO {
            numerator / denominator
        } else {
            Decimal::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_matrix() {
        let matrix = CorrelationMatrix::new();
        assert_eq!(matrix.size(), 0);
        assert!(matrix.get("EURUSD", "GBPUSD").is_none());
    }

    #[test]
    fn test_set_get_correlation() {
        let mut matrix = CorrelationMatrix::new();
        matrix.set("EURUSD", "GBPUSD", Decimal::from_f64(0.8).unwrap());

        assert_eq!(
            matrix.get("EURUSD", "GBPUSD"),
            Some(Decimal::from_f64(0.8).unwrap())
        );
        assert_eq!(
            matrix.get("GBPUSD", "EURUSD"),
            Some(Decimal::from_f64(0.8).unwrap())
        );
    }

    #[test]
    fn test_highest_correlation() {
        let mut matrix = CorrelationMatrix::new();
        matrix.set("EURUSD", "GBPUSD", Decimal::from_f64(0.8).unwrap());
        matrix.set("EURUSD", "USDJPY", Decimal::from_f64(-0.7).unwrap());
        matrix.set("EURUSD", "AUDUSD", Decimal::from_f64(0.3).unwrap());

        let (sym, corr) = matrix.highest_correlation("EURUSD").unwrap();
        assert_eq!(corr.abs(), Decimal::from_f64(0.8).unwrap());
    }

    #[test]
    fn test_currency_correlation() {
        let matrix = CorrelationMatrix::new();

        let corr = matrix.calculate_currency_correlation("EURUSD", "EURGBP");
        assert!(corr > Decimal::from_f64(0.5).unwrap());

        let corr = matrix.calculate_currency_correlation("EURUSD", "USDCHF");
        assert!(corr < Decimal::ZERO); // Inverse
    }

    #[test]
    fn test_rolling_correlation() {
        let mut rolling = RollingCorrelation::new(10);

        for i in 0..15 {
            rolling.add_observation("EURUSD", Decimal::from(i));
            rolling.add_observation("GBPUSD", Decimal::from(i + 1));
        }

        let corr = rolling.calculate("EURUSD", "GBPUSD");
        assert!(corr.is_some());
    }

    #[test]
    fn test_portfolio_risk() {
        let mut matrix = CorrelationMatrix::new();
        matrix.set("EURUSD", "GBPUSD", Decimal::from_f64(0.8).unwrap());

        let positions = vec![
            ("EURUSD".to_string(), Decimal::from(1), 1),
            ("GBPUSD".to_string(), Decimal::from(1), 1),
        ];

        let risk = matrix.portfolio_risk(&positions);
        assert!(risk > Decimal::ZERO);
    }

    #[test]
    fn test_remove_symbol() {
        let mut matrix = CorrelationMatrix::new();
        matrix.set("EURUSD", "GBPUSD", Decimal::from_f64(0.8).unwrap());

        matrix.remove_symbol("GBPUSD");

        assert!(!matrix.contains("GBPUSD"));
        assert!(matrix.get("EURUSD", "GBPUSD").is_none());
    }
}
