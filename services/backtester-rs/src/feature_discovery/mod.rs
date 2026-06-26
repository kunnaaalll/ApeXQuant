use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureType {
    VolatilityCompression,
    LiquidityImbalance,
    SpreadExpansion,
    RegimePersistence,
    SessionMomentum,
    OrderflowPressure,
}

#[derive(Debug, Clone)]
pub struct FeatureScore {
    pub feature_id: Uuid,
    pub score: Decimal,
    pub confidence: Decimal,
}

#[derive(Debug, Clone)]
pub struct FeatureImportance {
    pub feature_id: Uuid,
    pub strategy_id: Uuid,
    pub importance_weight: Decimal,
}

#[derive(Debug, Clone)]
pub struct FeatureDecay {
    pub feature_id: Uuid,
    pub decay_rate: Decimal,
    pub half_life_periods: u64,
}

pub trait FeatureEvaluator {
    fn evaluate(&self) -> FeatureScore;
}
