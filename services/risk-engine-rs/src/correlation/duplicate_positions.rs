//! Duplicate position detection
use rust_decimal::prelude::FromPrimitive;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of duplicate position check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCheckResult {
    /// Whether duplicates were found
    pub has_duplicates: bool,
    /// Duplicate position details
    pub duplicates: Vec<DuplicatePosition>,
    /// Total duplicate exposure
    pub duplicate_exposure: Decimal,
    /// Recommended action
    pub recommendation: DuplicateRecommendation,
}

/// A detected duplicate position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicatePosition {
    /// Symbol
    pub symbol: String,
    /// Number of positions
    pub position_count: u32,
    /// Total size across duplicates
    pub total_size: Decimal,
    /// Directions (can have opposing positions)
    pub directions: Vec<i8>,
}

/// Recommendation for handling duplicates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DuplicateRecommendation {
    /// No action needed
    None,
    /// Consider consolidating positions
    Consolidate,
    /// Closing duplicates recommended
    CloseDuplicates,
    /// Warning - manual review needed
    Review,
}

/// Detector for duplicate positions
pub struct DuplicatePositionDetector {
    /// Size threshold for considering positions as significant
    min_size_threshold: Decimal,
    /// Count threshold for duplicate warning
    warning_threshold: u32,
}

impl DuplicatePositionDetector {
    /// Create new detector
    pub fn new() -> Self {
        Self {
            min_size_threshold: Decimal::from_f64(0.01).unwrap(),
            warning_threshold: 2,
        }
    }

    /// Detect duplicate positions
    pub fn detect(
        &self,
        symbol: &str,
        direction: i8,
        positions: &[(String, Decimal, i8)],
    ) -> DuplicateCheckResult {
        let mut symbol_counts: HashMap<String, (u32, Decimal, Vec<i8>)> = HashMap::new();

        // Count existing positions
        for (pos_symbol, size, pos_direction) in positions {
            if *size >= self.min_size_threshold {
                let entry = symbol_counts
                    .entry(pos_symbol.clone())
                    .or_insert((0, Decimal::ZERO, Vec::new()));
                entry.0 += 1;
                entry.1 += size.abs();
                entry.2.push(*pos_direction);
            }
        }

        // Add the proposed position
        let entry = symbol_counts
            .entry(symbol.to_string())
            .or_insert((0, Decimal::ZERO, Vec::new()));
        entry.0 += 1;
        entry.1 += Decimal::ONE; // Normalized size for proposed
        entry.2.push(direction);

        // Find duplicates
        let mut duplicates = Vec::new();
        let mut total_duplicate_exposure = Decimal::ZERO;

        for (sym, (count, total_size, dirs)) in symbol_counts {
            if count >= self.warning_threshold {
                duplicates.push(DuplicatePosition {
                    symbol: sym.clone(),
                    position_count: count,
                    total_size,
                    directions: dirs.clone(),
                });

                // Count net exposure for same direction duplicates
                let same_direction_count = dirs.iter().filter(|&&d| d == direction).count() as u32;
                if same_direction_count >= self.warning_threshold {
                    total_duplicate_exposure += total_size;
                }
            }
        }

        let has_duplicates = !duplicates.is_empty();

        let recommendation = if duplicates.len() > 2 {
            DuplicateRecommendation::CloseDuplicates
        } else if has_duplicates {
            DuplicateRecommendation::Consolidate
        } else {
            DuplicateRecommendation::None
        };

        DuplicateCheckResult {
            has_duplicates,
            duplicates,
            duplicate_exposure: total_duplicate_exposure,
            recommendation,
        }
    }

    /// Check if positions would net out (hedge)
    pub fn is_hedged(&self, symbol: &str, positions: &[(String, Decimal, i8)]) -> bool {
        let symbol_positions: Vec<_> = positions
            .iter()
            .filter(|(s, _, _)| s == symbol)
            .collect();

        if symbol_positions.len() < 2 {
            return false;
        }

        let long_size: Decimal = symbol_positions
            .iter()
            .filter(|(_, _, d)| *d > 0)
            .map(|(_, s, _)| *s)
            .sum();

        let short_size: Decimal = symbol_positions
            .iter()
            .filter(|(_, _, d)| *d < 0)
            .map(|(_, s, _)| *s)
            .sum();

        // Hedged if we have opposing positions of similar size
        let min_size = long_size.min(short_size);
        let max_size = long_size.max(short_size);

        max_size > Decimal::ZERO && min_size / max_size > Decimal::from_f64(0.8).unwrap()
    }

    /// Calculate effective exposure accounting for hedges
    pub fn effective_exposure(&self, positions: &[(String, Decimal, i8)]) -> Decimal {
        let mut symbol_exposure: HashMap<String, (Decimal, Decimal)> = HashMap::new(); // (long, short)

        for (symbol, size, direction) in positions {
            let entry = symbol_exposure.entry(symbol.clone()).or_insert((Decimal::ZERO, Decimal::ZERO));

            if *direction > 0 {
                entry.0 += size.abs();
            } else {
                entry.1 += size.abs();
            }
        }

        symbol_exposure
            .values()
            .map(|(long, short)| (long - short).abs())
            .sum()
    }

    /// Get net direction for a symbol
    pub fn net_direction(&self, symbol: &str, positions: &[(String, Decimal, i8)]) -> i8 {
        let symbol_positions: Vec<_> = positions.iter().filter(|(s, _, _)| s == symbol).collect();

        let long_size: Decimal = symbol_positions
            .iter()
            .filter(|(_, _, d)| *d > 0)
            .map(|(_, s, _)| *s)
            .sum();

        let short_size: Decimal = symbol_positions
            .iter()
            .filter(|(_, _, d)| *d < 0)
            .map(|(_, s, _)| *s)
            .sum();

        if long_size > short_size {
            1
        } else if short_size > long_size {
            -1
        } else {
            0
        }
    }
}

impl Default for DuplicatePositionDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_duplicates() {
        let detector = DuplicatePositionDetector::new();
        let positions = vec![
            ("EURUSD".to_string(), Decimal::from(1), 1),
            ("GBPUSD".to_string(), Decimal::from(1), 1),
        ];

        let result = detector.detect("USDJPY", 1, &positions);
        assert!(!result.has_duplicates);
        assert_eq!(result.recommendation, DuplicateRecommendation::None);
    }

    #[test]
    fn test_duplicate_detected() {
        let detector = DuplicatePositionDetector::new();
        let positions = vec![
            ("EURUSD".to_string(), Decimal::from(1), 1),
            ("EURUSD".to_string(), Decimal::from(1), 1), // Duplicate
        ];

        let result = detector.detect("GBPUSD", 1, &positions);
        assert!(result.has_duplicates);
        assert_eq!(result.duplicates[0].symbol, "EURUSD");
    }

    #[test]
    fn test_hedged_detection() {
        let detector = DuplicatePositionDetector::new();
        let positions = vec![
            ("EURUSD".to_string(), Decimal::from(1), 1),
            ("EURUSD".to_string(), Decimal::from(1), -1), // Hedge
        ];

        assert!(detector.is_hedged("EURUSD", &positions));
    }

    #[test]
    fn test_effective_exposure() {
        let detector = DuplicatePositionDetector::new();
        let positions = vec![
            ("EURUSD".to_string(), Decimal::from(2), 1),
            ("EURUSD".to_string(), Decimal::from(1), -1), // Partial hedge
            ("GBPUSD".to_string(), Decimal::from(1), 1),
        ];

        let effective = detector.effective_exposure(&positions);
        // EURUSD net = 1, GBPUSD net = 1, total = 2
        assert_eq!(effective, Decimal::from(2));
    }

    #[test]
    fn test_net_direction() {
        let detector = DuplicatePositionDetector::new();
        let positions = vec![
            ("EURUSD".to_string(), Decimal::from(2), 1),
            ("EURUSD".to_string(), Decimal::from(1), -1),
        ];

        assert_eq!(detector.net_direction("EURUSD", &positions), 1);
    }
}
