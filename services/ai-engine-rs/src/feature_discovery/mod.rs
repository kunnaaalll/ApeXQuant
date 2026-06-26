use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureImportance {
    pub feature_name: String,
    pub importance_score: Decimal,
    pub statistical_significance: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDecay {
    pub half_life_periods: u32,
    pub decay_rate: Decimal,
    pub predictive_horizon: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStrength {
    pub raw_signal_strength: Decimal,
    pub noise_ratio: Decimal,
    pub signal_to_noise: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeDependency {
    pub optimal_regime: String,
    pub sensitivity: Decimal,
    pub cross_regime_stability: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredFeature {
    pub id: String,
    pub name: String,
    pub importance: Option<FeatureImportance>,
    pub decay: Option<FeatureDecay>,
    pub strength: Option<FeatureStrength>,
    pub regime_dependency: Option<RegimeDependency>,
}

pub struct FeatureDiscoveryEngine {}

impl FeatureDiscoveryEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate_volatility_compression(&self, _symbol: &str) -> DiscoveredFeature {
        DiscoveredFeature {
            id: format!("feat-vol-comp-{}", _symbol),
            name: String::from("volatility_compression"),
            importance: None,
            decay: None,
            strength: None,
            regime_dependency: None,
        }
    }

    pub fn evaluate_liquidity_imbalance(&self, _symbol: &str) -> DiscoveredFeature {
        DiscoveredFeature {
            id: format!("feat-liq-imb-{}", _symbol),
            name: String::from("liquidity_imbalance"),
            importance: None,
            decay: None,
            strength: None,
            regime_dependency: None,
        }
    }

    pub fn evaluate_momentum_exhaustion(&self, _symbol: &str) -> DiscoveredFeature {
        DiscoveredFeature {
            id: format!("feat-mom-exh-{}", _symbol),
            name: String::from("momentum_exhaustion"),
            importance: None,
            decay: None,
            strength: None,
            regime_dependency: None,
        }
    }
    
    pub fn evaluate_session_transitions(&self, _symbol: &str) -> DiscoveredFeature {
        DiscoveredFeature {
            id: format!("feat-sess-trans-{}", _symbol),
            name: String::from("session_transitions"),
            importance: None,
            decay: None,
            strength: None,
            regime_dependency: None,
        }
    }

    pub fn evaluate_spread_expansion(&self, _symbol: &str) -> DiscoveredFeature {
        DiscoveredFeature {
            id: format!("feat-spr-exp-{}", _symbol),
            name: String::from("spread_expansion"),
            importance: None,
            decay: None,
            strength: None,
            regime_dependency: None,
        }
    }

    pub fn evaluate_execution_degradation(&self, _symbol: &str) -> DiscoveredFeature {
        DiscoveredFeature {
            id: format!("feat-exec-deg-{}", _symbol),
            name: String::from("execution_degradation"),
            importance: None,
            decay: None,
            strength: None,
            regime_dependency: None,
        }
    }
}

impl Default for FeatureDiscoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}
