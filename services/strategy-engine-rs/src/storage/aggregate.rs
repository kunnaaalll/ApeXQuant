use serde::{Deserialize, Serialize};
use super::events::{StrategyEventWrapper, HealthEvent, ConfidenceEvent, DriftEvent, AllocationEvent, RecommendationEvent, DegradationEvent, MetaEvent, ClusterEvent, ContextEvent, ValidationEvent, ShadowEvent};
use super::rebuilder::Aggregatable;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StrategyAggregate {
    pub health_status: String,
    pub confidence_level: String,
    pub drift_status: String,
    pub allocation_status: String,
    pub recommendation_status: String,
    pub degradation_status: String,
    pub meta_status: String,
    pub cluster_status: String,
    pub context_status: String,
    pub validation_status: String,
    pub shadow_status: String,
    
    // Counters to ensure determinism
    pub health_updates: i32,
    pub confidence_updates: i32,
    pub drift_updates: i32,
    pub allocation_updates: i32,
    pub recommendation_updates: i32,
    pub degradation_updates: i32,
    pub meta_updates: i32,
    pub cluster_updates: i32,
    pub context_updates: i32,
    pub validation_updates: i32,
    pub shadow_updates: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StrategyAggregateSnapshot {
    pub health_status: String,
    pub confidence_level: String,
    pub drift_status: String,
    pub allocation_status: String,
    pub recommendation_status: String,
    pub degradation_status: String,
    pub meta_status: String,
    pub cluster_status: String,
    pub context_status: String,
    pub validation_status: String,
    pub shadow_status: String,

    pub health_updates: i32,
    pub confidence_updates: i32,
    pub drift_updates: i32,
    pub allocation_updates: i32,
    pub recommendation_updates: i32,
    pub degradation_updates: i32,
    pub meta_updates: i32,
    pub cluster_updates: i32,
    pub context_updates: i32,
    pub validation_updates: i32,
    pub shadow_updates: i32,
}

impl Aggregatable for StrategyAggregate {
    type Snapshot = StrategyAggregateSnapshot;
    type Error = String;

    fn apply_event(&mut self, event: &StrategyEventWrapper) -> Result<(), Self::Error> {
        match event {
            StrategyEventWrapper::Health(e) => {
                self.health_status = e.status.clone();
                self.health_updates += 1;
            }
            StrategyEventWrapper::Confidence(e) => {
                self.confidence_level = e.tier.clone();
                self.confidence_updates += 1;
            }
            StrategyEventWrapper::Drift(e) => {
                self.drift_status = e.requires_retraining.to_string();
                self.drift_updates += 1;
            }
            StrategyEventWrapper::Allocation(e) => {
                self.allocation_status = e.recommended_allocation.to_string();
                self.allocation_updates += 1;
            }
            StrategyEventWrapper::Recommendation(e) => {
                self.recommendation_status = e.action.clone();
                self.recommendation_updates += 1;
            }
            StrategyEventWrapper::Degradation(e) => {
                self.degradation_status = e.reason.clone();
                self.degradation_updates += 1;
            }
            StrategyEventWrapper::Meta(e) => {
                self.meta_status = e.adaptation_score.to_string();
                self.meta_updates += 1;
            }
            StrategyEventWrapper::Cluster(e) => {
                self.cluster_status = e.cluster_id.clone();
                self.cluster_updates += 1;
            }
            StrategyEventWrapper::Context(e) => {
                self.context_status = e.market_regime.clone();
                self.context_updates += 1;
            }
            StrategyEventWrapper::Validation(e) => {
                self.validation_status = e.is_valid.to_string();
                self.validation_updates += 1;
            }
            StrategyEventWrapper::Shadow(e) => {
                self.shadow_status = e.is_diverging.to_string();
                self.shadow_updates += 1;
            }
        }
        Ok(())
    }

    fn snapshot(&self) -> Self::Snapshot {
        StrategyAggregateSnapshot {
            health_status: self.health_status.clone(),
            confidence_level: self.confidence_level.clone(),
            drift_status: self.drift_status.clone(),
            allocation_status: self.allocation_status.clone(),
            recommendation_status: self.recommendation_status.clone(),
            degradation_status: self.degradation_status.clone(),
            meta_status: self.meta_status.clone(),
            cluster_status: self.cluster_status.clone(),
            context_status: self.context_status.clone(),
            validation_status: self.validation_status.clone(),
            shadow_status: self.shadow_status.clone(),

            health_updates: self.health_updates,
            confidence_updates: self.confidence_updates,
            drift_updates: self.drift_updates,
            allocation_updates: self.allocation_updates,
            recommendation_updates: self.recommendation_updates,
            degradation_updates: self.degradation_updates,
            meta_updates: self.meta_updates,
            cluster_updates: self.cluster_updates,
            context_updates: self.context_updates,
            validation_updates: self.validation_updates,
            shadow_updates: self.shadow_updates,
        }
    }

    fn restore(snapshot: Self::Snapshot) -> Result<Self, Self::Error> {
        Ok(StrategyAggregate {
            health_status: snapshot.health_status,
            confidence_level: snapshot.confidence_level,
            drift_status: snapshot.drift_status,
            allocation_status: snapshot.allocation_status,
            recommendation_status: snapshot.recommendation_status,
            degradation_status: snapshot.degradation_status,
            meta_status: snapshot.meta_status,
            cluster_status: snapshot.cluster_status,
            context_status: snapshot.context_status,
            validation_status: snapshot.validation_status,
            shadow_status: snapshot.shadow_status,

            health_updates: snapshot.health_updates,
            confidence_updates: snapshot.confidence_updates,
            drift_updates: snapshot.drift_updates,
            allocation_updates: snapshot.allocation_updates,
            recommendation_updates: snapshot.recommendation_updates,
            degradation_updates: snapshot.degradation_updates,
            meta_updates: snapshot.meta_updates,
            cluster_updates: snapshot.cluster_updates,
            context_updates: snapshot.context_updates,
            validation_updates: snapshot.validation_updates,
            shadow_updates: snapshot.shadow_updates,
        })
    }
}
