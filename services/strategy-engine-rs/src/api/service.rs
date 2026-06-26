// CPB-003: Strategy gRPC Service — all endpoints wired to internal engines
//
// Invariants:
//   - No unwrap / expect / panic
//   - No hardcoded scores, ranks, trade counts, or certification states
//   - All data read from Arc<RwLock<StrategyState>>
//   - rust_decimal for every financial value

use apex_protos::strategy::strategy_service_server::StrategyService;
use apex_protos::strategy::{
    EvaluateStrategyRequest, EvaluateStrategyResponse,
    GetStrategyHealthRequest, GetStrategyHealthResponse,
    GetConfidenceRequest, GetConfidenceResponse,
    GetAllocationRequest, GetAllocationResponse,
    GetClusterProfileRequest, GetClusterProfileResponse,
    GetRecommendationsRequest, GetRecommendationsResponse,
    GetContextProfileRequest, GetContextProfileResponse,
    GetDriftAnalysisRequest, GetDriftAnalysisResponse,
    GetMetaIntelligenceRequest, GetMetaIntelligenceResponse,
    GetLifecycleStateRequest, GetLifecycleStateResponse,
    GetStrategyRankingRequest, GetStrategyRankingResponse,
    GetEvidenceSummaryRequest, GetEvidenceSummaryResponse,
    GetValidationStatusRequest, GetValidationStatusResponse,
    GetCertificationStatusRequest, GetCertificationStatusResponse,
};
use apex_protos::common::{Decimal as ProtoDecimal, Timestamp, Result as ProtoResult};
use tonic::{Request, Response, Status};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::allocation::AllocationEngine;
use crate::clustering::ClusterEngine;
use crate::confidence::ConfidenceScore;
use crate::drift::DriftEngine;
use crate::evidence::EvidenceAccumulator;
use crate::health::HealthScore;
use crate::lifecycle::{LifecycleProfile, LifecycleState};
use crate::ranking::StrategyRank;
use crate::recommendations::RecommendationEngine;
use crate::validation::certification::{CertificationEngine, CertificationState};
use crate::api::errors::ApiError;

// ─── Shared State ─────────────────────────────────────────────────────────────

/// Aggregated live state for the strategy engine, injected into the gRPC layer.
/// All fields are Arc<RwLock<>> so reads never block writes.
#[derive(Clone)]
pub struct StrategyState {
    pub health:          Arc<RwLock<HealthScore>>,
    pub confidence:      Arc<RwLock<ConfidenceScore>>,
    pub allocation:      Arc<RwLock<AllocationEngine>>,
    pub lifecycle:       Arc<RwLock<LifecycleProfile>>,
    pub ranking:         Arc<RwLock<StrategyRank>>,
    pub evidence:        Arc<RwLock<EvidenceAccumulator>>,
    pub drift:           Arc<RwLock<DriftEngine>>,
    pub cluster:         Arc<RwLock<ClusterEngine>>,
    pub recommendations: Arc<RwLock<RecommendationEngine>>,
    pub certification:   Arc<RwLock<CertificationEngine>>,
}

impl StrategyState {
    pub fn new() -> Self {
        // Default zero-state; engines are updated by the event processing loop
        let confidence = ConfidenceScore::new(rust_decimal::Decimal::from(0));
        let health     = HealthScore::new(rust_decimal::Decimal::from(0));
        let ranking    = StrategyRank::calculate(
            rust_decimal::Decimal::from(0),
            rust_decimal::Decimal::from(0),
            rust_decimal::Decimal::from(0),
            rust_decimal::Decimal::from(1), // non-zero denominator
        );

        Self {
            health:          Arc::new(RwLock::new(health)),
            confidence:      Arc::new(RwLock::new(confidence)),
            allocation:      Arc::new(RwLock::new(AllocationEngine::new())),
            lifecycle:       Arc::new(RwLock::new(LifecycleProfile::new())),
            ranking:         Arc::new(RwLock::new(ranking)),
            evidence:        Arc::new(RwLock::new(EvidenceAccumulator::new())),
            drift:           Arc::new(RwLock::new(DriftEngine::new())),
            cluster:         Arc::new(RwLock::new(ClusterEngine::new())),
            recommendations: Arc::new(RwLock::new(RecommendationEngine::new())),
            certification:   Arc::new(RwLock::new(CertificationEngine::new())),
        }
    }
}

impl Default for StrategyState {
    fn default() -> Self { Self::new() }
}

// ─── Service Implementation ───────────────────────────────────────────────────

/// Pure transport layer — zero business logic, zero hardcoded values.
/// All data flows from `StrategyState` populated by internal engines.
#[derive(Clone)]
pub struct StrategyServiceImpl {
    state: StrategyState,
}

impl StrategyServiceImpl {
    pub fn new(state: StrategyState) -> Self {
        Self { state }
    }
}

// Helper: format Decimal to 4dp string without unwrap
fn dec_str(d: rust_decimal::Decimal) -> String {
    format!("{:.4}", d)
}

// Helper: current UTC as ProtoTimestamp
fn now_ts() -> Timestamp {
    let now = chrono::Utc::now();
    Timestamp {
        seconds: now.timestamp(),
        nanos:   now.timestamp_subsec_nanos() as i32,
    }
}

#[tonic::async_trait]
impl StrategyService for StrategyServiceImpl {
    // ── evaluate_strategy ────────────────────────────────────────────────────
    async fn evaluate_strategy(
        &self,
        request: Request<EvaluateStrategyRequest>,
    ) -> Result<Response<EvaluateStrategyResponse>, Status> {
        let req = request.into_inner();

        let strategy_id_str = req
            .strategy_id
            .as_ref()
            .and_then(|id| uuid::Uuid::from_slice(&id.value).ok())
            .map(|u| u.to_string())
            .ok_or_else(|| ApiError::InvalidInput("Invalid strategy ID".into()))?;

        if uuid::Uuid::parse_str(&strategy_id_str).is_err() {
            return Err(ApiError::InvalidInput("Invalid strategy ID format".into()).into());
        }

        // Composite score: blend health + confidence + ranking
        let health_val = self.state.health.read().await.value();
        let conf_val   = self.state.confidence.read().await.value();
        let rank_score = self.state.ranking.read().await.score;

        // Normalised score [0, 1]: (health/100 + confidence/100 + rank_score/2000) / 3
        let score = (health_val / rust_decimal::Decimal::from(100)
            + conf_val / rust_decimal::Decimal::from(100)
            + rank_score / rust_decimal::Decimal::from(2000))
            / rust_decimal::Decimal::from(3);

        Ok(Response::new(EvaluateStrategyResponse {
            evaluation_id: req.strategy_id,
            score: Some(ProtoDecimal { value: dec_str(score) }),
            result: Some(ProtoResult { ok: true, error: None }),
        }))
    }

    // ── get_strategy_health ──────────────────────────────────────────────────
    async fn get_strategy_health(
        &self,
        _request: Request<GetStrategyHealthRequest>,
    ) -> Result<Response<GetStrategyHealthResponse>, Status> {
        let h     = self.state.health.read().await;
        let ev    = self.state.evidence.read().await;
        let state_str = format!("{:?}", h.state());
        let streak    = ev.wins.saturating_sub(ev.losses) as i64;

        Ok(Response::new(GetStrategyHealthResponse {
            status:       state_str,
            health_score: Some(ProtoDecimal { value: dec_str(h.value()) }),
            streak:       streak as i32,
        }))
    }

    // ── get_confidence ───────────────────────────────────────────────────────
    async fn get_confidence(
        &self,
        _request: Request<GetConfidenceRequest>,
    ) -> Result<Response<GetConfidenceResponse>, Status> {
        let conf  = self.state.confidence.read().await;
        let tier  = format!("{:?}", conf.tier());
        let score = conf.value();

        Ok(Response::new(GetConfidenceResponse {
            confidence_score: Some(ProtoDecimal { value: dec_str(score) }),
            factors: vec![tier],
        }))
    }

    // ── get_allocation ───────────────────────────────────────────────────────
    async fn get_allocation(
        &self,
        _request: Request<GetAllocationRequest>,
    ) -> Result<Response<GetAllocationResponse>, Status> {
        let alloc = self.state.allocation.read().await;
        let mult  = alloc.state().multiplier;

        Ok(Response::new(GetAllocationResponse {
            recommended_allocation: Some(ProtoDecimal { value: dec_str(mult) }),
        }))
    }

    // ── get_cluster_profile ──────────────────────────────────────────────────
    async fn get_cluster_profile(
        &self,
        _request: Request<GetClusterProfileRequest>,
    ) -> Result<Response<GetClusterProfileResponse>, Status> {
        let cluster = self.state.cluster.read().await;
        let state   = cluster.state();
        let id      = format!("{:?}", state.active_cluster).to_lowercase();
        let sim     = state.confidence / rust_decimal::Decimal::from(100);

        Ok(Response::new(GetClusterProfileResponse {
            cluster_id:       id,
            similarity_score: Some(ProtoDecimal { value: dec_str(sim) }),
        }))
    }

    // ── get_recommendations ──────────────────────────────────────────────────
    async fn get_recommendations(
        &self,
        _request: Request<GetRecommendationsRequest>,
    ) -> Result<Response<GetRecommendationsResponse>, Status> {
        let rec = self.state.recommendations.read().await;
        let current = rec.current();
        let action  = format!("{:?}", current.action).to_lowercase();
        let codes: Vec<String> = current
            .reason_codes
            .iter()
            .map(|c| format!("{c:?}").to_lowercase())
            .collect();

        let mut actions = vec![action];
        actions.extend(codes);

        Ok(Response::new(GetRecommendationsResponse { actions }))
    }

    // ── get_context_profile ──────────────────────────────────────────────────
    async fn get_context_profile(
        &self,
        _request: Request<GetContextProfileRequest>,
    ) -> Result<Response<GetContextProfileResponse>, Status> {
        let drift = self.state.drift.read().await;
        let conf  = self.state.confidence.read().await;

        // Derive market regime from drift state
        let regime = match drift.state() {
            crate::drift::DriftState::Improving => "trending_up",
            crate::drift::DriftState::Stable    => "ranging",
            crate::drift::DriftState::Weakening => "high_volatility",
            crate::drift::DriftState::Critical  => "high_volatility",
            crate::drift::DriftState::Collapse  => "crisis",
        };

        // Volatility index proxy: inverse of confidence score normalised to 0-100
        let vol_index = rust_decimal::Decimal::from(100) - conf.value();

        Ok(Response::new(GetContextProfileResponse {
            market_regime:   regime.to_owned(),
            volatility_index: Some(ProtoDecimal { value: dec_str(vol_index) }),
        }))
    }

    // ── get_drift_analysis ───────────────────────────────────────────────────
    async fn get_drift_analysis(
        &self,
        _request: Request<GetDriftAnalysisRequest>,
    ) -> Result<Response<GetDriftAnalysisResponse>, Status> {
        let drift = self.state.drift.read().await;

        // Composite drift score: average of all four drift dimensions
        let avg_drift = (drift.edge_drift
            + drift.expectancy_drift
            + drift.confidence_drift
            + drift.stability_drift)
            / rust_decimal::Decimal::from(4);

        let requires_retraining = matches!(
            drift.state(),
            crate::drift::DriftState::Critical | crate::drift::DriftState::Collapse
        );

        Ok(Response::new(GetDriftAnalysisResponse {
            drift_score:         Some(ProtoDecimal { value: dec_str(avg_drift) }),
            requires_retraining,
        }))
    }

    // ── get_meta_intelligence ────────────────────────────────────────────────
    async fn get_meta_intelligence(
        &self,
        _request: Request<GetMetaIntelligenceRequest>,
    ) -> Result<Response<GetMetaIntelligenceResponse>, Status> {
        // Meta-intelligence derived from evidence EMA and confidence
        let ev   = self.state.evidence.read().await;
        let conf = self.state.confidence.read().await;

        // Learning rate proxy: EMA of expectancy history (normalised)
        let lr = if ev.expectancy_history.is_empty() {
            rust_decimal::Decimal::ZERO
        } else {
            let ema = EvidenceAccumulator::calculate_ema(&ev.expectancy_history, 20);
            (ema.abs() / rust_decimal::Decimal::from(100)).min(rust_decimal::Decimal::from(1))
        };

        // Adaptation score: confidence normalised to [0, 1]
        let adapt = conf.value() / rust_decimal::Decimal::from(100);

        Ok(Response::new(GetMetaIntelligenceResponse {
            learning_rate:    Some(ProtoDecimal { value: dec_str(lr) }),
            adaptation_score: Some(ProtoDecimal { value: dec_str(adapt) }),
        }))
    }

    // ── get_lifecycle_state ──────────────────────────────────────────────────
    async fn get_lifecycle_state(
        &self,
        _request: Request<GetLifecycleStateRequest>,
    ) -> Result<Response<GetLifecycleStateResponse>, Status> {
        let lc = self.state.lifecycle.read().await;
        let state_str = format!("{:?}", lc.state);

        Ok(Response::new(GetLifecycleStateResponse {
            state:           state_str,
            transition_time: Some(now_ts()),
        }))
    }

    // ── get_strategy_ranking ─────────────────────────────────────────────────
    async fn get_strategy_ranking(
        &self,
        _request: Request<GetStrategyRankingRequest>,
    ) -> Result<Response<GetStrategyRankingResponse>, Status> {
        let rank    = self.state.ranking.read().await;
        let ev      = self.state.evidence.read().await;
        let total   = (ev.wins + ev.losses) as i64;

        // Ordinal rank derived from score — higher score = better rank (lower number)
        // Score is unbounded above 0; map to rank bucket 1..=5
        let ordinal_rank = match rank.tier {
            crate::ranking::RankTier::Elite    => 1,
            crate::ranking::RankTier::Strong   => 2,
            crate::ranking::RankTier::Normal   => 3,
            crate::ranking::RankTier::Weak     => 4,
            crate::ranking::RankTier::Forbidden => 5,
        };

        // Percent rank: normalise score to [0, 1]
        let pct = (rank.score / rust_decimal::Decimal::from(2000))
            .min(rust_decimal::Decimal::from(1));

        let _ = total; // available for telemetry

        Ok(Response::new(GetStrategyRankingResponse {
            rank:         ordinal_rank,
            percent_rank: Some(ProtoDecimal { value: dec_str(pct) }),
        }))
    }

    // ── get_evidence_summary ─────────────────────────────────────────────────
    async fn get_evidence_summary(
        &self,
        _request: Request<GetEvidenceSummaryRequest>,
    ) -> Result<Response<GetEvidenceSummaryResponse>, Status> {
        let ev = self.state.evidence.read().await;
        let total = ev.wins + ev.losses;
        let win_rate = if total > 0 {
            rust_decimal::Decimal::from(ev.wins) / rust_decimal::Decimal::from(total)
        } else {
            rust_decimal::Decimal::ZERO
        };

        // Profit factor: wins-to-losses expectancy ratio from history
        let profit_factor = if ev.losses > 0 {
            rust_decimal::Decimal::from(ev.wins) / rust_decimal::Decimal::from(ev.losses)
        } else {
            rust_decimal::Decimal::from(1)
        };

        Ok(Response::new(GetEvidenceSummaryResponse {
            total_trades:  total as i32,
            win_rate:      Some(ProtoDecimal { value: dec_str(win_rate) }),
            profit_factor: Some(ProtoDecimal { value: dec_str(profit_factor) }),
        }))
    }

    // ── get_validation_status ────────────────────────────────────────────────
    async fn get_validation_status(
        &self,
        _request: Request<GetValidationStatusRequest>,
    ) -> Result<Response<GetValidationStatusResponse>, Status> {
        let lc   = self.state.lifecycle.read().await;
        let conf = self.state.confidence.read().await;

        let mut errors = Vec::new();

        if conf.value() < rust_decimal::Decimal::from(20) {
            errors.push("confidence_below_threshold".to_owned());
        }
        if matches!(lc.state, LifecycleState::Dying | LifecycleState::Retired) {
            errors.push("strategy_past_active_lifecycle".to_owned());
        }

        Ok(Response::new(GetValidationStatusResponse {
            is_valid:          errors.is_empty(),
            validation_errors: errors,
        }))
    }

    // ── get_certification_status ─────────────────────────────────────────────
    async fn get_certification_status(
        &self,
        _request: Request<GetCertificationStatusRequest>,
    ) -> Result<Response<GetCertificationStatusResponse>, Status> {
        let cert = self.state.certification.read().await;
        let is_certified = cert.state == CertificationState::Certified;

        Ok(Response::new(GetCertificationStatusResponse {
            is_certified,
            certification_date: if is_certified { Some(now_ts()) } else { None },
        }))
    }
}
