use uuid::Uuid;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningEvent {
    StrategyDiscovered { strategy_id: Uuid, blueprint_hash: String },
    EdgeDecay { strategy_id: Uuid, decay_rate: rust_decimal::Decimal },
    NewFeature { feature_id: Uuid, feature_name: String },
    StrategyRetired { strategy_id: Uuid, reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsumedLearningData {
    RegimeMemory { regime_id: Uuid, confidence: rust_decimal::Decimal },
    Anomaly { anomaly_id: Uuid, market: String },
    Recommendation { target_id: Uuid, action: String },
}

pub trait LearningPublisher {
    fn publish(&self, event: LearningEvent) -> Result<(), &'static str>;
}

pub trait LearningConsumer {
    fn consume(&mut self, data: ConsumedLearningData) -> Result<(), &'static str>;
}

pub struct DummyLearningBroker;

impl LearningPublisher for DummyLearningBroker {
    fn publish(&self, _event: LearningEvent) -> Result<(), &'static str> {
        Ok(())
    }
}

impl LearningConsumer for DummyLearningBroker {
    fn consume(&mut self, _data: ConsumedLearningData) -> Result<(), &'static str> {
        Ok(())
    }
}
