//! Synthetic exposure detection
use rust_decimal::prelude::FromPrimitive;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Synthetic exposure detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticExposure {
    /// Detected synthetic positions
    pub positions: Vec<String>,

    /// Leverage amplification factor
    pub leverage_factor: Decimal,

    /// Hidden position created
    pub hidden_position: Option<String>,

    /// Risk multiplier
    pub risk_multiplier: Decimal,
}

/// Analyzes positions for synthetic/hidden exposure
pub struct SyntheticExposureAnalyzer {
    /// Known synthetic relationships
    relationships: HashMap<String, Vec<String>>,
}

impl SyntheticExposureAnalyzer {
    /// Create new analyzer with predefined relationships
    pub fn new() -> Self {
        let mut relationships = HashMap::new();

        // EURUSD + USDCHF = EURCHF (synthetic)
        relationships.insert(
            "EURCHF".to_string(),
            vec!["EURUSD".to_string(), "USDCHF".to_string()],
        );

        // EURUSD + USDJPY = EURJPY
        relationships.insert(
            "EURJPY".to_string(),
            vec!["EURUSD".to_string(), "USDJPY".to_string()],
        );

        // GBPUSD + USDJPY = GBPJPY
        relationships.insert(
            "GBPJPY".to_string(),
            vec!["GBPUSD".to_string(), "USDJPY".to_string()],
        );

        // EURGBP + GBPUSD = EURUSD
        relationships.insert(
            "EURUSD".to_string(),
            vec!["EURGBP".to_string(), "GBPUSD".to_string()],
        );

        // Commodity crosses
        relationships.insert(
            "AUDNZD".to_string(),
            vec!["AUDUSD".to_string(), "NZDUSD".to_string()],
        );

        Self { relationships }
    }

    /// Detect synthetic exposures from current positions
    pub fn detect(&self, new_symbol: &str, positions: &[(String, Decimal, i8)]) -> SyntheticExposure {
        let mut detected_positions = Vec::new();
        let mut leverage_factor = Decimal::ONE;

        // Get all held symbols
        let held_symbols: HashSet<_> = positions
            .iter()
            .map(|(s, _, _)| s.as_str())
            .chain(std::iter::once(new_symbol))
            .collect();

        // Check for synthetic positions
        for (synthetic, components) in &self.relationships {
            // Check if we have all components to create a synthetic
            let has_all_components = components
                .iter()
                .all(|comp| held_symbols.contains(comp.as_str()));

            if has_all_components && !held_symbols.contains(synthetic.as_str()) {
                detected_positions.push(format!("Synthetic {} from {:?}", synthetic, components));

                // Synthetic position increases effective leverage
                leverage_factor += Decimal::from_f64(0.3).unwrap_or(Decimal::ONE);
            }
        }

        // Check for triangular arbitrage positions (unusual)
        let triangular = self.detect_triangular_exposure(positions);
        if triangular {
            detected_positions.push("Triangular exposure detected".to_string());
            leverage_factor *= Decimal::from_f64(1.5).unwrap_or(Decimal::ONE);
        }

        // Check for overleverage through multiple USD positions
        let usd_leverage = self.check_usd_overleverage(positions, new_symbol);
        if usd_leverage > Decimal::from(2) {
            detected_positions.push(format!("USD leverage: {:.1}x", usd_leverage));
        }

        let risk_multiplier = if leverage_factor > Decimal::from(2) {
            Decimal::from_f64(0.5).unwrap_or(Decimal::ONE) // Reduce size
        } else if leverage_factor > Decimal::from_f64(1.3).unwrap() {
            Decimal::from_f64(0.75).unwrap_or(Decimal::ONE)
        } else {
            Decimal::ONE
        };

        SyntheticExposure {
            positions: detected_positions,
            leverage_factor,
            hidden_position: None,
            risk_multiplier,
        }
    }

    /// Check if adding a position would create problematic synthetic
    pub fn would_create_synthetic(&self, symbol: &str, positions: &[(String, Decimal, i8)]) -> bool {
        let held_symbols: HashSet<_> = positions.iter().map(|(s, _, _)| s.as_str()).collect();

        for (synthetic, components) in &self.relationships {
            if synthetic == symbol {
                // Adding the synthetic itself - check if components exist
                let has_components = components
                    .iter()
                    .all(|comp| held_symbols.contains(comp.as_str()));
                if has_components {
                    return true;
                }
            }

            if components.contains(&symbol.to_string()) {
                // Adding a component - check if we have all others
                let missing: Vec<_> = components
                    .iter()
                    .filter(|c| c.as_str() != symbol && !held_symbols.contains(c.as_str()))
                    .collect();

                if missing.is_empty() {
                    return true;
                }
            }
        }

        false
    }

    /// Calculate hidden leverage from position correlations
    pub fn hidden_leverage(&self, positions: &[(String, Decimal, i8)]) -> Decimal {
        if positions.len() < 2 {
            return Decimal::ONE;
        }

        let total_absolute: Decimal = positions.iter().map(|(_, s, _)| s.abs()).sum();
        let net_exposure = positions.iter().map(|(_, s, d)| *s * Decimal::from(*d as i32)).sum::<Decimal>().abs();

        if net_exposure > Decimal::ZERO {
            total_absolute / net_exposure
        } else {
            Decimal::ONE
        }
    }

    fn detect_triangular_exposure(&self, positions: &[(String, Decimal, i8)]) -> bool {
        // Simplified: check for 3+ positions that form a triangle
        let symbol_count = positions.len();

        // More than 3 correlated positions suggests triangular
        symbol_count >= 3 && self.count_common_currencies(positions) <= 2
    }

    fn count_common_currencies(&self, positions: &[(String, Decimal, i8)]) -> usize {
        let mut currencies = HashSet::new();

        for (symbol, _, _) in positions {
            if symbol.len() >= 6 {
                currencies.insert(&symbol[0..3]);
                currencies.insert(&symbol[3..6]);
            }
        }

        currencies.len()
    }

    fn check_usd_overleverage(
        &self,
        positions: &[(String, Decimal, i8)],
        new_symbol: &str,
    ) -> Decimal {
        let mut usd_notional = Decimal::ZERO;

        for (symbol, size, _) in positions {
            if symbol.contains("USD") {
                usd_notional += size.abs();
            }
        }

        if new_symbol.contains("USD") {
            usd_notional += Decimal::ONE;
        }

        usd_notional
    }
}

impl Default for SyntheticExposureAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_synthetic_empty() {
        let analyzer = SyntheticExposureAnalyzer::new();
        let exposure = analyzer.detect("EURUSD", &[]);

        assert!(exposure.positions.is_empty());
        assert_eq!(exposure.leverage_factor, Decimal::ONE);
    }

    #[test]
    fn test_synthetic_detection() {
        let analyzer = SyntheticExposureAnalyzer::new();

        // EURUSD + USDCHF creates synthetic EURCHF
        let positions = vec![
            ("EURUSD".to_string(), Decimal::from(1), 1),
            ("USDCHF".to_string(), Decimal::from(1), 1),
        ];

        let exposure = analyzer.detect("GBPUSD", &positions);
        assert!(!exposure.positions.is_empty());
    }

    #[test]
    fn test_hidden_leverage() {
        let analyzer = SyntheticExposureAnalyzer::new();

        let positions = vec![
            ("EURUSD".to_string(), Decimal::from(1), 1),
            ("EURUSD".to_string(), Decimal::from(1), -1), // Hedge
        ];

        let leverage = analyzer.hidden_leverage(&positions);
        assert!(leverage > Decimal::ONE);
    }

    #[test]
    fn test_would_create_synthetic() {
        let analyzer = SyntheticExposureAnalyzer::new();

        let positions = vec![
            ("EURUSD".to_string(), Decimal::from(1), 1),
            ("USDCHF".to_string(), Decimal::from(1), 1),
        ];

        assert!(analyzer.would_create_synthetic("EURCHF", &positions));
    }
}
