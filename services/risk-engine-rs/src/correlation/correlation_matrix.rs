use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    /// Dimension name (e.g., Symbol, Currency, Sector, Theme) -> (Entity A, Entity B) -> Correlation
    /// To maintain symmetry, we always store keys ordered: (min(A, B), max(A, B))
    matrices: BTreeMap<String, BTreeMap<(String, String), Decimal>>,
}

impl Default for CorrelationMatrix {
    fn default() -> Self {
        Self::new()
    }
}

impl CorrelationMatrix {
    pub fn new() -> Self {
        Self {
            matrices: BTreeMap::new(),
        }
    }

    /// Sets the correlation between two entities in a specific dimension.
    /// The correlation value is strictly bounded between -1.0 and 1.0.
    pub fn set_correlation(
        &mut self,
        dimension: &str,
        entity_a: &str,
        entity_b: &str,
        mut correlation: Decimal,
    ) {
        let neg_one = Decimal::new(-1, 0);
        let pos_one = Decimal::new(1, 0);

        if correlation < neg_one {
            correlation = neg_one;
        } else if correlation > pos_one {
            correlation = pos_one;
        }

        let key = Self::ordered_key(entity_a, entity_b);
        let dim_map = self.matrices.entry(dimension.to_string()).or_default();
        dim_map.insert(key, correlation);
    }

    /// Retrieves the correlation between two entities in a specific dimension.
    /// Returns 1.0 if entity_a == entity_b.
    /// Returns 0.0 if there is no known correlation.
    pub fn get_correlation(&self, dimension: &str, entity_a: &str, entity_b: &str) -> Decimal {
        if entity_a == entity_b {
            return Decimal::new(1, 0); // Diagonal is always 1.0
        }

        let key = Self::ordered_key(entity_a, entity_b);
        self.matrices
            .get(dimension)
            .and_then(|dim_map| dim_map.get(&key))
            .copied()
            .unwrap_or_else(|| Decimal::new(0, 0))
    }

    /// Retrieves all correlations for a specific entity in a specific dimension.
    pub fn get_correlations_for(&self, dimension: &str, entity: &str) -> Vec<(String, Decimal)> {
        let mut results = Vec::new();
        if let Some(dim_map) = self.matrices.get(dimension) {
            for ((a, b), &corr) in dim_map.iter() {
                if a == entity {
                    results.push((b.clone(), corr));
                } else if b == entity {
                    results.push((a.clone(), corr));
                }
            }
        }
        results
    }

    /// Ensures A and B are ordered to maintain a symmetric representation
    fn ordered_key(a: &str, b: &str) -> (String, String) {
        if a <= b {
            (a.to_string(), b.to_string())
        } else {
            (b.to_string(), a.to_string())
        }
    }

    pub fn get_dimensions(&self) -> Vec<String> {
        self.matrices.keys().cloned().collect()
    }

    pub fn get_dimension_keys(&self, dimension: &str) -> Vec<(String, String)> {
        self.matrices.get(dimension).map(|m| m.keys().cloned().collect()).unwrap_or_default()
    }
}
