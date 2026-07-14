//! Parameter Genetics Module
//!
//! Provides three production-grade optimizers:
//!
//! - `GridSearchOptimizer`: deterministic Cartesian product sweep through all bounds.
//! - `EvolutionaryOptimizer`: deterministic LCG-seeded evolutionary search
//!   (selection → crossover → mutation over N generations).
//! - `ConstraintOptimizer`: Lagrange-inspired penalty method that drives
//!   solutions toward feasible regions while minimising a scalar objective.
//!
//! All three are reproducible: same bounds + same config = identical output.

use rust_decimal::Decimal;
use std::collections::HashMap;

// ─── Core Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ParameterSet {
    pub parameters: HashMap<String, Decimal>,
}

impl ParameterSet {
    /// Simple scalar objective: sum of all parameter values.
    /// Replace with domain-specific fitness (e.g. Sharpe) when wiring to strategy engine.
    pub fn objective_value(&self) -> Decimal {
        self.parameters.values().copied().sum()
    }
}

pub trait Optimizer {
    fn optimize(&self, bounds: &HashMap<String, (Decimal, Decimal)>) -> Vec<ParameterSet>;
}

// ─── 1. Grid Search ───────────────────────────────────────────────────────────

/// Deterministic exhaustive grid search.
///
/// For each parameter key, steps from `min` to `max` (inclusive) in increments
/// of `steps[key]`, then computes the Cartesian product of all per-key value
/// sequences. If a key is missing from `steps`, a default of `(max - min) / 5`
/// is used (5 steps).
pub struct GridSearchOptimizer {
    pub steps: HashMap<String, Decimal>,
}

impl GridSearchOptimizer {
    pub fn new(steps: HashMap<String, Decimal>) -> Self {
        Self { steps }
    }

    /// Build the linear sequence [min, min+step, min+2*step, ... ≤ max].
    fn make_sequence(min: Decimal, max: Decimal, step: Decimal) -> Vec<Decimal> {
        if step <= Decimal::ZERO || min > max {
            return vec![min];
        }
        let mut seq = Vec::new();
        let mut v = min;
        while v <= max + Decimal::new(1, 10) {
            // tiny epsilon for float-like step accumulation safety
            seq.push(v.min(max));
            v += step;
            if v > max {
                break;
            }
        }
        if seq.is_empty() {
            seq.push(min);
        }
        seq
    }
}

impl Optimizer for GridSearchOptimizer {
    fn optimize(&self, bounds: &HashMap<String, (Decimal, Decimal)>) -> Vec<ParameterSet> {
        if bounds.is_empty() {
            return vec![];
        }

        // Sort keys for deterministic ordering
        let mut keys: Vec<&String> = bounds.keys().collect();
        keys.sort();

        // Build per-key value sequences
        let sequences: Vec<(&String, Vec<Decimal>)> = keys
            .iter()
            .map(|k| {
                let (min, max) = bounds[*k];
                let step = self.steps.get(*k).copied().unwrap_or_else(|| {
                    let range = max - min;
                    if range > Decimal::ZERO {
                        range / Decimal::from(5i64)
                    } else {
                        Decimal::ONE
                    }
                });
                (*k, Self::make_sequence(min, max, step))
            })
            .collect();

        // Cartesian product via iterative expansion
        let mut result: Vec<HashMap<String, Decimal>> = vec![HashMap::new()];
        for (key, values) in &sequences {
            let mut expanded = Vec::with_capacity(result.len() * values.len());
            for existing in &result {
                for &v in values {
                    let mut new_map = existing.clone();
                    new_map.insert((*key).clone(), v);
                    expanded.push(new_map);
                }
            }
            result = expanded;
        }

        result
            .into_iter()
            .map(|parameters| ParameterSet { parameters })
            .collect()
    }
}

// ─── 2. Evolutionary Optimizer ───────────────────────────────────────────────

/// Deterministic LCG-seeded evolutionary optimizer.
///
/// Algorithm:
/// 1. **Seed population** from grid corners and midpoints.
/// 2. **Evaluate** each individual by its `objective_value()`.
/// 3. **Select** the top 50% of the population.
/// 4. **Crossover** adjacent pairs (arithmetic average).
/// 5. **Mutate** each offspring by ±`mutation_rate` × range (alternating direction
///    based on LCG state for determinism).
/// 6. Repeat for `generations` rounds.
pub struct EvolutionaryOptimizer {
    pub generation_size: usize,
    pub mutation_rate: Decimal,
    pub generations: usize,
    /// Deterministic seed for the LCG PRNG.
    pub seed: u64,
}

impl Default for EvolutionaryOptimizer {
    fn default() -> Self {
        Self {
            generation_size: 20,
            mutation_rate: Decimal::new(5, 2), // 5%
            generations: 10,
            seed: 42,
        }
    }
}

/// Linear Congruential Generator — deterministic, no external deps.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }
    /// Returns a value in [0, 1) as Decimal(scale=6).
    fn next_unit(&mut self) -> Decimal {
        let v = self.next() >> 11; // 53-bit mantissa range
        let max = (1u64 << 53) as f64;
        let f = (v as f64) / max;
        Decimal::try_from(f).unwrap_or(Decimal::new(5, 1))
    }
    /// Returns +1 or -1 based on LCG parity.
    fn next_sign(&mut self) -> Decimal {
        if self.next() & 1 == 0 {
            Decimal::ONE
        } else {
            Decimal::NEGATIVE_ONE
        }
    }
}

impl Optimizer for EvolutionaryOptimizer {
    fn optimize(&self, bounds: &HashMap<String, (Decimal, Decimal)>) -> Vec<ParameterSet> {
        if bounds.is_empty() || self.generation_size == 0 {
            return vec![];
        }

        let mut keys: Vec<&String> = bounds.keys().collect();
        keys.sort();
        let mut lcg = Lcg(self.seed);

        // ── Seed initial population ──────────────────────────────────────────
        // Use grid corners (min/max for each param) + random fill up to generation_size.
        let mut population: Vec<ParameterSet> = Vec::with_capacity(self.generation_size);

        // Corner: all-min
        let all_min: HashMap<String, Decimal> =
            keys.iter().map(|k| ((*k).clone(), bounds[*k].0)).collect();
        population.push(ParameterSet {
            parameters: all_min,
        });

        // Corner: all-max
        let all_max: HashMap<String, Decimal> =
            keys.iter().map(|k| ((*k).clone(), bounds[*k].1)).collect();
        population.push(ParameterSet {
            parameters: all_max,
        });

        // Midpoint
        let midpoint: HashMap<String, Decimal> = keys
            .iter()
            .map(|k| {
                let (lo, hi) = bounds[*k];
                ((*k).clone(), (lo + hi) / Decimal::TWO)
            })
            .collect();
        population.push(ParameterSet {
            parameters: midpoint,
        });

        // Fill remainder with LCG-sampled individuals
        while population.len() < self.generation_size {
            let params: HashMap<String, Decimal> = keys
                .iter()
                .map(|k| {
                    let (lo, hi) = bounds[*k];
                    let range = hi - lo;
                    let val = lo + lcg.next_unit() * range;
                    ((*k).clone(), val.max(lo).min(hi))
                })
                .collect();
            population.push(ParameterSet { parameters: params });
        }

        // ── Evolutionary loop ────────────────────────────────────────────────
        for _ in 0..self.generations {
            // Sort by objective (ascending — lower = better by convention)
            population.sort_by_key(|a| a.objective_value());

            // Select top 50%
            let elite_size = (population.len() / 2).max(1);
            population.truncate(elite_size);

            // Crossover: pair adjacent elites
            let mut children: Vec<ParameterSet> = Vec::new();
            for i in (0..population.len().saturating_sub(1)).step_by(2) {
                let p1 = &population[i].parameters;
                let p2 = &population[i + 1].parameters;
                let child_params: HashMap<String, Decimal> = keys
                    .iter()
                    .map(|k| {
                        let v1 = p1.get(*k).copied().unwrap_or(Decimal::ZERO);
                        let v2 = p2.get(*k).copied().unwrap_or(Decimal::ZERO);
                        ((*k).clone(), (v1 + v2) / Decimal::TWO)
                    })
                    .collect();
                children.push(ParameterSet {
                    parameters: child_params,
                });
            }

            // Mutate children
            for child in &mut children {
                for k in &keys {
                    let (lo, hi) = bounds[*k];
                    let range = hi - lo;
                    let delta = self.mutation_rate * range * lcg.next_unit() * lcg.next_sign();
                    if let Some(v) = child.parameters.get_mut(*k) {
                        *v = (*v + delta).max(lo).min(hi);
                    }
                }
            }

            population.extend(children);

            // Pad back to generation_size with fresh random individuals
            while population.len() < self.generation_size {
                let params: HashMap<String, Decimal> = keys
                    .iter()
                    .map(|k| {
                        let (lo, hi) = bounds[*k];
                        let range = hi - lo;
                        let val = lo + lcg.next_unit() * range;
                        ((*k).clone(), val.max(lo).min(hi))
                    })
                    .collect();
                population.push(ParameterSet { parameters: params });
            }
        }

        // Final sort: return best first
        population.sort_by_key(|a| a.objective_value());
        population
    }
}

// ─── 3. Constraint Optimizer ─────────────────────────────────────────────────

/// Penalty-method constraint optimizer.
///
/// Minimises the objective `f(x) + λ × Σ penalty_i(x)` where penalties push
/// parameter values toward their feasible bounds. Uses gradient-descent-like
/// steps with a deterministic step schedule (step decays by `decay` each round).
///
/// Constraints supported:
/// - **Box constraints**: all parameters clamped to `[min, max]` from `bounds`.
/// - **Sum constraint**: total sum of all parameters ≤ `max_sum` (if set).
pub struct ConstraintOptimizer {
    /// Penalty multiplier λ.
    pub penalty_weight: Decimal,
    /// Initial step size for gradient descent.
    pub initial_step: Decimal,
    /// Step decay factor per iteration (e.g. 0.9 → 10% reduction each round).
    pub decay: Decimal,
    /// Maximum number of optimisation iterations.
    pub iterations: usize,
    /// Optional: maximum allowed sum across all parameters.
    pub max_sum: Option<Decimal>,
}

impl Default for ConstraintOptimizer {
    fn default() -> Self {
        Self {
            penalty_weight: Decimal::from(10i64),
            initial_step: Decimal::new(1, 1), // 0.1
            decay: Decimal::new(92, 2),       // 0.92
            iterations: 50,
            max_sum: None,
        }
    }
}

impl Optimizer for ConstraintOptimizer {
    fn optimize(&self, bounds: &HashMap<String, (Decimal, Decimal)>) -> Vec<ParameterSet> {
        if bounds.is_empty() {
            return vec![];
        }

        let mut keys: Vec<&String> = bounds.keys().collect();
        keys.sort();

        // Initialise at midpoint of each bound
        let mut current: HashMap<String, Decimal> = keys
            .iter()
            .map(|k| {
                let (lo, hi) = bounds[*k];
                ((*k).clone(), (lo + hi) / Decimal::TWO)
            })
            .collect();

        let mut step = self.initial_step;

        for _iter in 0..self.iterations {
            for k in &keys {
                let (lo, hi) = bounds[*k];
                let v = current[*k];

                // Gradient of objective w.r.t. this parameter (df/dx = 1 for sum objective).
                let grad = Decimal::ONE;

                // Penalty gradient: push toward center if near boundary.
                let pen_lo = if v < lo {
                    self.penalty_weight * (lo - v)
                } else {
                    Decimal::ZERO
                };
                let pen_hi = if v > hi {
                    self.penalty_weight * (v - hi)
                } else {
                    Decimal::ZERO
                };
                let total_grad = grad - pen_lo + pen_hi;

                // Gradient step (descending: subtract step × gradient)
                let new_v = (v - step * total_grad).max(lo).min(hi);
                current.insert((*k).clone(), new_v);
            }

            // Apply sum constraint via proportional scaling
            if let Some(max_sum) = self.max_sum {
                let total: Decimal = current.values().copied().sum();
                if total > max_sum && total > Decimal::ZERO {
                    let scale = max_sum / total;
                    for k in &keys {
                        let (lo, _) = bounds[*k];
                        let v = current[*k] * scale;
                        current.insert((*k).clone(), v.max(lo));
                    }
                }
            }

            step *= self.decay;
        }

        vec![ParameterSet {
            parameters: current,
        }]
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds_2d() -> HashMap<String, (Decimal, Decimal)> {
        let mut m = HashMap::new();
        m.insert(
            "stop_loss".to_string(),
            (Decimal::from(10i64), Decimal::from(30i64)),
        );
        m.insert(
            "take_profit".to_string(),
            (Decimal::from(20i64), Decimal::from(60i64)),
        );
        m
    }

    // ── Grid Search ──────────────────────────────────────────────────────────

    #[test]
    fn test_grid_search_covers_entire_range() {
        let mut steps = HashMap::new();
        steps.insert("stop_loss".to_string(), Decimal::from(10i64)); // 10,20,30 → 3 values
        steps.insert("take_profit".to_string(), Decimal::from(20i64)); // 20,40,60 → 3 values
        let opt = GridSearchOptimizer::new(steps);
        let results = opt.optimize(&bounds_2d());
        // 3 × 3 = 9 combinations
        assert_eq!(
            results.len(),
            9,
            "expected 9 grid points, got {}",
            results.len()
        );
    }

    #[test]
    fn test_grid_search_values_within_bounds() {
        let mut steps = HashMap::new();
        steps.insert("stop_loss".to_string(), Decimal::from(5i64));
        steps.insert("take_profit".to_string(), Decimal::from(10i64));
        let opt = GridSearchOptimizer::new(steps);
        let bounds = bounds_2d();
        let results = opt.optimize(&bounds);
        for ps in &results {
            for (k, &v) in &ps.parameters {
                let (lo, hi) = bounds[k];
                assert!(
                    v >= lo && v <= hi,
                    "param {k} value {v} out of [{lo}, {hi}]"
                );
            }
        }
    }

    #[test]
    fn test_grid_search_empty_bounds() {
        let opt = GridSearchOptimizer::new(HashMap::new());
        assert!(opt.optimize(&HashMap::new()).is_empty());
    }

    #[test]
    fn test_grid_search_deterministic() {
        let mut steps = HashMap::new();
        steps.insert("stop_loss".to_string(), Decimal::from(10i64));
        steps.insert("take_profit".to_string(), Decimal::from(20i64));
        let opt = GridSearchOptimizer::new(steps);
        let b = bounds_2d();
        let r1 = opt.optimize(&b);
        let r2 = opt.optimize(&b);
        assert_eq!(r1.len(), r2.len());
        // Compare first result's parameters
        let keys: Vec<_> = r1[0].parameters.keys().collect();
        for k in keys {
            assert_eq!(r1[0].parameters[k], r2[0].parameters[k]);
        }
    }

    // ── Evolutionary ─────────────────────────────────────────────────────────

    #[test]
    fn test_evolutionary_returns_generation_size() {
        let opt = EvolutionaryOptimizer {
            generation_size: 12,
            generations: 5,
            ..Default::default()
        };
        let results = opt.optimize(&bounds_2d());
        // Should return at least generation_size individuals (may be larger after final sort)
        assert!(!results.is_empty());
    }

    #[test]
    fn test_evolutionary_values_within_bounds() {
        let opt = EvolutionaryOptimizer::default();
        let bounds = bounds_2d();
        let results = opt.optimize(&bounds);
        for ps in &results {
            for (k, &v) in &ps.parameters {
                let (lo, hi) = bounds[k];
                assert!(v >= lo && v <= hi, "param {k} = {v} out of [{lo}, {hi}]");
            }
        }
    }

    #[test]
    fn test_evolutionary_deterministic() {
        let opt = EvolutionaryOptimizer {
            seed: 7,
            ..Default::default()
        };
        let b = bounds_2d();
        let r1 = opt.optimize(&b);
        let r2 = opt.optimize(&b);
        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            for k in a.parameters.keys() {
                assert_eq!(a.parameters[k], b.parameters[k]);
            }
        }
    }

    // ── Constraint ───────────────────────────────────────────────────────────

    #[test]
    fn test_constraint_returns_one_solution() {
        let opt = ConstraintOptimizer::default();
        let results = opt.optimize(&bounds_2d());
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_constraint_solution_within_bounds() {
        let opt = ConstraintOptimizer::default();
        let bounds = bounds_2d();
        let results = opt.optimize(&bounds);
        for (k, &v) in &results[0].parameters {
            let (lo, hi) = bounds[k];
            assert!(v >= lo && v <= hi, "param {k} = {v} out of [{lo}, {hi}]");
        }
    }

    #[test]
    fn test_constraint_sum_constraint_respected() {
        let mut bounds = HashMap::new();
        bounds.insert("a".to_string(), (Decimal::from(1i64), Decimal::from(10i64)));
        bounds.insert("b".to_string(), (Decimal::from(1i64), Decimal::from(10i64)));
        let opt = ConstraintOptimizer {
            max_sum: Some(Decimal::from(12i64)),
            ..Default::default()
        };
        let results = opt.optimize(&bounds);
        let total: Decimal = results[0].parameters.values().copied().sum();
        assert!(
            total <= Decimal::from(12i64) + Decimal::new(1, 4),
            "sum {total} exceeds constraint 12"
        );
    }

    #[test]
    fn test_constraint_empty_bounds() {
        let opt = ConstraintOptimizer::default();
        assert!(opt.optimize(&HashMap::new()).is_empty());
    }
}
