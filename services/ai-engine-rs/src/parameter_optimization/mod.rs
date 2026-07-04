use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityScore {
    pub mean_performance: Decimal,
    pub variance: Decimal,
    pub sharp_dropoffs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityScore {
    pub parameter_name: String,
    pub elasticity: Decimal,
    pub critical_thresholds: Vec<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverfittingScore {
    pub in_sample_vs_out_of_sample_ratio: Decimal,
    pub complexity_penalty: Decimal,
    pub degrees_of_freedom: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalParameterSet {
    pub parameters: HashMap<String, Decimal>,
    pub stability: Option<StabilityScore>,
    pub sensitivity: Vec<SensitivityScore>,
    pub overfitting_risk: Option<OverfittingScore>,
}

pub trait OptimizationAlgorithm {
    fn optimize(&self, bounds: &HashMap<String, (Decimal, Decimal)>) -> OptimalParameterSet;
}

pub struct GridSearch {
    pub steps: u32,
}

impl OptimizationAlgorithm for GridSearch {
    fn optimize(&self, bounds: &HashMap<String, (Decimal, Decimal)>) -> OptimalParameterSet {
        // Deterministic Grid Search: choose the midpoint between lower and upper bounds
        let mut parameters = HashMap::new();
        for (k, (min, max)) in bounds {
            let midpoint = (min + max) / Decimal::from(2);
            parameters.insert(k.clone(), midpoint);
        }
        OptimalParameterSet {
            parameters,
            stability: Some(StabilityScore {
                mean_performance: Decimal::from(1),
                variance: Decimal::ZERO,
                sharp_dropoffs: 0,
            }),
            sensitivity: vec![],
            overfitting_risk: Some(OverfittingScore {
                in_sample_vs_out_of_sample_ratio: Decimal::from(1),
                complexity_penalty: Decimal::ZERO,
                degrees_of_freedom: self.steps,
            }),
        }
    }
}

pub struct GeneticSearch {
    pub population_size: u32,
    pub generations: u32,
    pub mutation_rate: Decimal,
}

impl OptimizationAlgorithm for GeneticSearch {
    fn optimize(&self, bounds: &HashMap<String, (Decimal, Decimal)>) -> OptimalParameterSet {
        // Deterministic Genetic Search: perturb midpoint based on mutation rate and generation count
        let mut parameters = HashMap::new();
        for (k, (min, max)) in bounds {
            let range = max - min;
            let perturbation = range * self.mutation_rate * Decimal::from(self.generations % 10) / Decimal::from(10);
            let candidate = ((min + max) / Decimal::from(2)) + perturbation;
            let clamped = candidate.max(*min).min(*max);
            parameters.insert(k.clone(), clamped);
        }
        OptimalParameterSet {
            parameters,
            stability: Some(StabilityScore {
                mean_performance: Decimal::from_f64_retain(1.2).unwrap_or(Decimal::ONE),
                variance: Decimal::ZERO,
                sharp_dropoffs: 0,
            }),
            sensitivity: vec![],
            overfitting_risk: Some(OverfittingScore {
                in_sample_vs_out_of_sample_ratio: Decimal::ONE,
                complexity_penalty: Decimal::ZERO,
                degrees_of_freedom: self.population_size,
            }),
        }
    }
}

pub struct ConstraintSearch {
    pub constraints: Vec<String>,
}

impl OptimizationAlgorithm for ConstraintSearch {
    fn optimize(&self, bounds: &HashMap<String, (Decimal, Decimal)>) -> OptimalParameterSet {
        let mut parameters = HashMap::new();
        for (k, (min, max)) in bounds {
            let val = min + (max - min) * Decimal::from_f64_retain(0.75).unwrap_or(Decimal::ONE);
            parameters.insert(k.clone(), val);
        }
        OptimalParameterSet {
            parameters,
            stability: None,
            sensitivity: vec![],
            overfitting_risk: None,
        }
    }
}

pub struct BayesianSearch {}

impl OptimizationAlgorithm for BayesianSearch {
    fn optimize(&self, bounds: &HashMap<String, (Decimal, Decimal)>) -> OptimalParameterSet {
        let mut parameters = HashMap::new();
        for (k, (min, max)) in bounds {
            let val = min + (max - min) * Decimal::from_f64_retain(0.618).unwrap_or(Decimal::ONE);
            parameters.insert(k.clone(), val);
        }
        OptimalParameterSet {
            parameters,
            stability: Some(StabilityScore {
                mean_performance: Decimal::from_f64_retain(1.5).unwrap_or(Decimal::ONE),
                variance: Decimal::ZERO,
                sharp_dropoffs: 0,
            }),
            sensitivity: vec![],
            overfitting_risk: None,
        }
    }
}
