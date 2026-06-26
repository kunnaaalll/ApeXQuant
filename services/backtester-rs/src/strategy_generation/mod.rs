use rust_decimal::Decimal;
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyFamily {
    Momentum,
    MeanReversion,
    Breakout,
    VolatilityExpansion,
    CorrelationDivergence,
    SessionRotation,
    RegimeSwitching,
}

#[derive(Debug, Clone)]
pub struct RequiredFeatures {
    pub features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParameterTemplate {
    pub bounds: HashMap<String, (Decimal, Decimal)>,
    pub step_sizes: HashMap<String, Decimal>,
}

#[derive(Debug, Clone)]
pub struct StrategyBlueprint {
    pub id: Uuid,
    pub family: StrategyFamily,
    pub required_features: RequiredFeatures,
    pub parameter_template: ParameterTemplate,
}

pub trait StrategyGenerator {
    fn generate(&self) -> StrategyBlueprint;
}

pub struct MomentumGenerator;

impl StrategyGenerator for MomentumGenerator {
    fn generate(&self) -> StrategyBlueprint {
        StrategyBlueprint {
            id: Uuid::new_v4(),
            family: StrategyFamily::Momentum,
            required_features: RequiredFeatures { features: vec!["price_momentum".to_string()] },
            parameter_template: ParameterTemplate {
                bounds: HashMap::new(),
                step_sizes: HashMap::new(),
            },
        }
    }
}
