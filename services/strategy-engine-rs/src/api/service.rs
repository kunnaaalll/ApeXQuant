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
use apex_protos::common::{Decimal, Timestamp, Result as ProtoResult};
use tonic::{Request, Response, Status};
use uuid::Uuid;
use std::str::FromStr;

use crate::api::errors::ApiError;

/// The pure, stateless transport layer implementation for StrategyService.
/// Contains NO business logic, calculations, or state mutations.
#[derive(Default)]
pub struct StrategyServiceImpl {
    // In a real implementation, this would hold references to the internal engines or dispatchers.
    // Since Phase 8 is purely about the stateless transport layer, we will mock the responses
    // or return default values until the integration with the core engines is complete.
}

#[tonic::async_trait]
impl StrategyService for StrategyServiceImpl {
    async fn evaluate_strategy(
        &self,
        request: Request<EvaluateStrategyRequest>,
    ) -> Result<Response<EvaluateStrategyResponse>, Status> {
        let req = request.into_inner();
        
        let strategy_id_str = req.strategy_id.as_ref().map(|id| uuid::Uuid::from_slice(&id.value).unwrap_or_default().to_string()).unwrap_or_default();
        if Uuid::from_str(&strategy_id_str).is_err() {
            return Err(ApiError::InvalidInput("Invalid strategy ID".to_string()).into());
        }

        // Mock deterministic evaluation response
        let response = EvaluateStrategyResponse {
            evaluation_id: req.strategy_id, // Echoing back for simplicity in mock
            score: Some(Decimal { value: "0.85".to_string() }),
            result: Some(ProtoResult {
                ok: true,
                error: None,
            }),
        };

        Ok(Response::new(response))
    }

    async fn get_strategy_health(
        &self,
        request: Request<GetStrategyHealthRequest>,
    ) -> Result<Response<GetStrategyHealthResponse>, Status> {
        let _req = request.into_inner();
        let response = GetStrategyHealthResponse {
            status: "HEALTHY".to_string(),
            health_score: Some(Decimal { value: "0.92".to_string() }),
            streak: 5,
        };
        Ok(Response::new(response))
    }

    async fn get_confidence(
        &self,
        request: Request<GetConfidenceRequest>,
    ) -> Result<Response<GetConfidenceResponse>, Status> {
        let _req = request.into_inner();
        let response = GetConfidenceResponse {
            confidence_score: Some(Decimal { value: "0.78".to_string() }),
            factors: vec!["high_win_rate".to_string(), "stable_drawdown".to_string()],
        };
        Ok(Response::new(response))
    }

    async fn get_allocation(
        &self,
        request: Request<GetAllocationRequest>,
    ) -> Result<Response<GetAllocationResponse>, Status> {
        let _req = request.into_inner();
        let response = GetAllocationResponse {
            recommended_allocation: Some(Decimal { value: "0.15".to_string() }),
        };
        Ok(Response::new(response))
    }

    async fn get_cluster_profile(
        &self,
        request: Request<GetClusterProfileRequest>,
    ) -> Result<Response<GetClusterProfileResponse>, Status> {
        let _req = request.into_inner();
        let response = GetClusterProfileResponse {
            cluster_id: "momentum_alpha".to_string(),
            similarity_score: Some(Decimal { value: "0.95".to_string() }),
        };
        Ok(Response::new(response))
    }

    async fn get_recommendations(
        &self,
        request: Request<GetRecommendationsRequest>,
    ) -> Result<Response<GetRecommendationsResponse>, Status> {
        let _req = request.into_inner();
        let response = GetRecommendationsResponse {
            actions: vec!["reduce_exposure".to_string()],
        };
        Ok(Response::new(response))
    }

    async fn get_context_profile(
        &self,
        request: Request<GetContextProfileRequest>,
    ) -> Result<Response<GetContextProfileResponse>, Status> {
        let _req = request.into_inner();
        let response = GetContextProfileResponse {
            market_regime: "high_volatility".to_string(),
            volatility_index: Some(Decimal { value: "24.5".to_string() }),
        };
        Ok(Response::new(response))
    }

    async fn get_drift_analysis(
        &self,
        request: Request<GetDriftAnalysisRequest>,
    ) -> Result<Response<GetDriftAnalysisResponse>, Status> {
        let _req = request.into_inner();
        let response = GetDriftAnalysisResponse {
            drift_score: Some(Decimal { value: "0.12".to_string() }),
            requires_retraining: false,
        };
        Ok(Response::new(response))
    }

    async fn get_meta_intelligence(
        &self,
        request: Request<GetMetaIntelligenceRequest>,
    ) -> Result<Response<GetMetaIntelligenceResponse>, Status> {
        let _req = request.into_inner();
        let response = GetMetaIntelligenceResponse {
            learning_rate: Some(Decimal { value: "0.01".to_string() }),
            adaptation_score: Some(Decimal { value: "0.88".to_string() }),
        };
        Ok(Response::new(response))
    }

    async fn get_lifecycle_state(
        &self,
        request: Request<GetLifecycleStateRequest>,
    ) -> Result<Response<GetLifecycleStateResponse>, Status> {
        let _req = request.into_inner();
        let response = GetLifecycleStateResponse {
            state: "ACTIVE".to_string(),
            transition_time: Some(Timestamp {
                seconds: 1680000000,
                nanos: 0,
            }),
        };
        Ok(Response::new(response))
    }

    async fn get_strategy_ranking(
        &self,
        request: Request<GetStrategyRankingRequest>,
    ) -> Result<Response<GetStrategyRankingResponse>, Status> {
        let _req = request.into_inner();
        let response = GetStrategyRankingResponse {
            rank: 12,
            percent_rank: Some(Decimal { value: "0.91".to_string() }),
        };
        Ok(Response::new(response))
    }

    async fn get_evidence_summary(
        &self,
        request: Request<GetEvidenceSummaryRequest>,
    ) -> Result<Response<GetEvidenceSummaryResponse>, Status> {
        let _req = request.into_inner();
        let response = GetEvidenceSummaryResponse {
            total_trades: 154,
            win_rate: Some(Decimal { value: "0.62".to_string() }),
            profit_factor: Some(Decimal { value: "1.45".to_string() }),
        };
        Ok(Response::new(response))
    }

    async fn get_validation_status(
        &self,
        request: Request<GetValidationStatusRequest>,
    ) -> Result<Response<GetValidationStatusResponse>, Status> {
        let _req = request.into_inner();
        let response = GetValidationStatusResponse {
            is_valid: true,
            validation_errors: vec![],
        };
        Ok(Response::new(response))
    }

    async fn get_certification_status(
        &self,
        request: Request<GetCertificationStatusRequest>,
    ) -> Result<Response<GetCertificationStatusResponse>, Status> {
        let _req = request.into_inner();
        let response = GetCertificationStatusResponse {
            is_certified: true,
            certification_date: Some(Timestamp {
                seconds: 1680000000,
                nanos: 0,
            }),
        };
        Ok(Response::new(response))
    }
}
