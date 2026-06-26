use uuid::Uuid;
use crate::research_lab::ResearchJob;
use crate::parameter_genetics::ParameterSet;
use crate::strategy_generation::StrategyBlueprint;
use crate::feature_discovery::FeatureScore;

pub trait AIEngineInterface {
    fn request_research_job(&self, target_market: &str) -> ResearchJob;
    fn request_parameter_search(&self, blueprint_id: Uuid) -> Vec<ParameterSet>;
    fn request_feature_discovery(&self, data_segment_id: Uuid) -> Vec<FeatureScore>;
    fn request_strategy_generation(&self, parameters: &ParameterSet) -> StrategyBlueprint;
}

pub struct DummyAIEngine;

impl AIEngineInterface for DummyAIEngine {
    fn request_research_job(&self, _target_market: &str) -> ResearchJob {
        unimplemented!("AI Engine not yet connected")
    }

    fn request_parameter_search(&self, _blueprint_id: Uuid) -> Vec<ParameterSet> {
        unimplemented!("AI Engine not yet connected")
    }

    fn request_feature_discovery(&self, _data_segment_id: Uuid) -> Vec<FeatureScore> {
        unimplemented!("AI Engine not yet connected")
    }

    fn request_strategy_generation(&self, _parameters: &ParameterSet) -> StrategyBlueprint {
        unimplemented!("AI Engine not yet connected")
    }
}
