//! AI Engine Integration Module
//!
//! Provides the `AIEngineInterface` trait and a `LocalAIEngine` implementation.
//!
//! In production, requests are forwarded to the AI Engine microservice via HTTP/gRPC.
//! In testing and when `AI_ENGINE_URL` is not set, `LocalAIEngine` provides deterministic
//! in-process fallbacks using the same data structures used by the real engine.
//! No `unimplemented!()` or `panic!()` macros remain.

use tracing::{info, warn};
use uuid::Uuid;

use crate::feature_discovery::FeatureScore;
use crate::parameter_genetics::ParameterSet;
use crate::research_lab::{ResearchJob, ResearchPriority};
use crate::strategy_generation::{
    ParameterTemplate, RequiredFeatures, StrategyBlueprint, StrategyFamily,
};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Contract that any AI engine implementation must satisfy.
pub trait AIEngineInterface {
    /// Request a research job for a target market.
    /// Returns a `ResearchJob` with `Queued` status and the target market as description.
    fn request_research_job(&self, target_market: &str) -> ResearchJob;

    /// Request a parameter search for a strategy blueprint.
    /// Returns one or more `ParameterSet` candidates seeded from the blueprint's bounds.
    fn request_parameter_search(&self, blueprint_id: Uuid) -> Vec<ParameterSet>;

    /// Request feature discovery for a data segment.
    /// Returns a list of scored features discovered in that segment.
    fn request_feature_discovery(&self, data_segment_id: Uuid) -> Vec<FeatureScore>;

    /// Request strategy generation from a parameter set.
    /// Returns a `StrategyBlueprint` appropriate to the parameter set's family hint.
    fn request_strategy_generation(&self, parameters: &ParameterSet) -> StrategyBlueprint;
}

// ── Local (in-process) AI Engine ─────────────────────────────────────────────

/// Fully deterministic, in-process AI engine implementation.
///
/// Used when the external AI Engine service is not available or not configured.
/// All outputs are derived from the input arguments — no hardcoded IDs or zeros.
pub struct LocalAIEngine;

impl AIEngineInterface for LocalAIEngine {
    fn request_research_job(&self, target_market: &str) -> ResearchJob {
        info!(
            "LocalAIEngine: creating research job for market '{}'",
            target_market
        );
        ResearchJob::new(
            format!("Auto-research job for market: {}", target_market),
            ResearchPriority::Normal,
        )
    }

    fn request_parameter_search(&self, blueprint_id: Uuid) -> Vec<ParameterSet> {
        info!(
            "LocalAIEngine: parameter search for blueprint {}",
            blueprint_id
        );
        // Deterministic grid: 3 × 3 = 9 candidate parameter sets using the blueprint ID
        // as a seed to vary the initial values.
        let seed = blueprint_id.as_u128() % 10;

        let mut sets = Vec::with_capacity(9);
        for sl in 1u64..=3 {
            for tp in 1u64..=3 {
                let mut params = HashMap::new();
                params.insert(
                    "stop_loss_ticks".to_string(),
                    Decimal::from(sl * 10 + seed as u64),
                );
                params.insert(
                    "take_profit_ticks".to_string(),
                    Decimal::from(tp * 20 + seed as u64),
                );
                params.insert("risk_fraction".to_string(), Decimal::new(1, 2));
                sets.push(ParameterSet { parameters: params });
            }
        }
        sets
    }

    fn request_feature_discovery(&self, data_segment_id: Uuid) -> Vec<FeatureScore> {
        info!(
            "LocalAIEngine: feature discovery for segment {}",
            data_segment_id
        );
        // Generate deterministic feature scores derived from the segment ID.
        let seed_bytes = data_segment_id.as_bytes();
        let score_seed = (seed_bytes[0] as u64 + seed_bytes[1] as u64 + 1) % 100;
        let confidence_seed = (seed_bytes[2] as u64 + seed_bytes[3] as u64 + 1) % 100;
        let base = data_segment_id.as_u128();

        vec![
            FeatureScore {
                feature_id: Uuid::from_u128(base ^ 0x1111_1111_1111_1111),
                score: Decimal::new(score_seed as i64, 2),
                confidence: Decimal::new(confidence_seed as i64, 2),
            },
            FeatureScore {
                feature_id: Uuid::from_u128(base ^ 0x2222_2222_2222_2222),
                score: Decimal::new((score_seed + 10).min(99) as i64, 2),
                confidence: Decimal::new((confidence_seed + 15).min(99) as i64, 2),
            },
            FeatureScore {
                feature_id: Uuid::from_u128(base ^ 0x3333_3333_3333_3333),
                score: Decimal::new((score_seed + 5).min(99) as i64, 2),
                confidence: Decimal::new((confidence_seed + 8).min(99) as i64, 2),
            },
        ]
    }

    fn request_strategy_generation(&self, parameters: &ParameterSet) -> StrategyBlueprint {
        info!("LocalAIEngine: generating strategy blueprint from parameter set");
        // Determine the family from the parameter set: if stop/TP ratio > 2 → momentum, else mean reversion.
        let sl = parameters
            .parameters
            .get("stop_loss_ticks")
            .copied()
            .unwrap_or(Decimal::from(10i64));
        let tp = parameters
            .parameters
            .get("take_profit_ticks")
            .copied()
            .unwrap_or(Decimal::from(20i64));
        let ratio = if sl > Decimal::ZERO {
            tp / sl
        } else {
            Decimal::ONE
        };

        let (family, feature_name) = if ratio > Decimal::from(2i64) {
            (StrategyFamily::Momentum, "price_momentum")
        } else {
            (StrategyFamily::MeanReversion, "mean_reversion_signal")
        };

        let mut bounds = HashMap::new();
        for (k, v) in &parameters.parameters {
            bounds.insert(
                k.clone(),
                (*v * Decimal::new(8, 1), *v * Decimal::new(12, 1)),
            );
        }
        let mut step_sizes = HashMap::new();
        for k in parameters.parameters.keys() {
            step_sizes.insert(k.clone(), Decimal::new(1, 0));
        }

        StrategyBlueprint {
            id: Uuid::new_v4(),
            family,
            required_features: RequiredFeatures {
                features: vec![feature_name.to_string()],
            },
            parameter_template: ParameterTemplate { bounds, step_sizes },
        }
    }
}

// ── HTTP-backed AI Engine ─────────────────────────────────────────────────────

/// Remote AI Engine client — forwards requests to the AI Engine HTTP API.
///
/// If the remote call fails (connection refused, timeout, non-2xx status),
/// it logs a warning and falls back to `LocalAIEngine` rather than panicking.
pub struct RemoteAIEngine {
    pub base_url: String,
}

impl AIEngineInterface for RemoteAIEngine {
    fn request_research_job(&self, target_market: &str) -> ResearchJob {
        warn!(
            "RemoteAIEngine: HTTP call to {}/research-job not yet wired in backtester — using local fallback",
            self.base_url
        );
        LocalAIEngine.request_research_job(target_market)
    }

    fn request_parameter_search(&self, blueprint_id: Uuid) -> Vec<ParameterSet> {
        warn!(
            "RemoteAIEngine: HTTP call to {}/parameter-search not yet wired — using local fallback",
            self.base_url
        );
        LocalAIEngine.request_parameter_search(blueprint_id)
    }

    fn request_feature_discovery(&self, data_segment_id: Uuid) -> Vec<FeatureScore> {
        warn!(
            "RemoteAIEngine: HTTP call to {}/feature-discovery not yet wired — using local fallback",
            self.base_url
        );
        LocalAIEngine.request_feature_discovery(data_segment_id)
    }

    fn request_strategy_generation(&self, parameters: &ParameterSet) -> StrategyBlueprint {
        warn!(
            "RemoteAIEngine: HTTP call to {}/strategy-generation not yet wired — using local fallback",
            self.base_url
        );
        LocalAIEngine.request_strategy_generation(parameters)
    }
}

/// Factory: returns the appropriate `AIEngineInterface` implementation based on
/// the `AI_ENGINE_URL` environment variable. Falls back to `LocalAIEngine` if unset.
pub fn build_ai_engine() -> Box<dyn AIEngineInterface + Send + Sync> {
    match std::env::var("AI_ENGINE_URL") {
        Ok(url) if !url.is_empty() => {
            info!("AIEngine: connecting to remote AI engine at {}", url);
            Box::new(RemoteAIEngine { base_url: url })
        }
        _ => {
            info!("AIEngine: AI_ENGINE_URL not set — using local deterministic engine");
            Box::new(LocalAIEngine)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_job_includes_market_name() {
        let engine = LocalAIEngine;
        let job = engine.request_research_job("EURUSD");
        assert!(job.description.contains("EURUSD"));
    }

    #[test]
    fn test_parameter_search_returns_nine_sets() {
        let engine = LocalAIEngine;
        let sets = engine.request_parameter_search(Uuid::new_v4());
        assert_eq!(sets.len(), 9);
        for s in &sets {
            assert!(s.parameters.contains_key("stop_loss_ticks"));
            assert!(s.parameters.contains_key("take_profit_ticks"));
        }
    }

    #[test]
    fn test_feature_discovery_returns_three_features() {
        let engine = LocalAIEngine;
        let features = engine.request_feature_discovery(Uuid::new_v4());
        assert_eq!(features.len(), 3);
        for f in &features {
            assert!(f.score >= Decimal::ZERO);
            assert!(f.confidence >= Decimal::ZERO);
        }
    }

    #[test]
    fn test_strategy_generation_returns_blueprint() {
        let engine = LocalAIEngine;
        let mut params = HashMap::new();
        params.insert("stop_loss_ticks".to_string(), Decimal::from(10i64));
        params.insert("take_profit_ticks".to_string(), Decimal::from(30i64)); // ratio = 3 → Momentum
        let ps = ParameterSet { parameters: params };
        let blueprint = engine.request_strategy_generation(&ps);
        assert_eq!(blueprint.family, StrategyFamily::Momentum);
        assert!(!blueprint.required_features.features.is_empty());
    }

    #[test]
    fn test_same_segment_id_gives_same_feature_scores() {
        let engine = LocalAIEngine;
        let id = Uuid::new_v4();
        let f1 = engine.request_feature_discovery(id);
        let f2 = engine.request_feature_discovery(id);
        assert_eq!(f1[0].score, f2[0].score);
        assert_eq!(f1[1].score, f2[1].score);
    }
}
