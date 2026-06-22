use apex_protos::execution::execution_service_server::ExecutionService;
use apex_protos::execution::{
    EvaluateExecutionRequest, EvaluateExecutionResponse, ExecutionRiskRequest,
    ExecutionRiskResponse, GetOrderStateRequest, GetOrderStateResponse, GetPositionStateRequest,
    GetPositionStateResponse, HealthRequest as GrpcHealthRequest, HealthResponse as GrpcHealthResponse,
    LatencyRequest, LatencyResponse, LiquidityProfileRequest, LiquidityProfileResponse,
    MicrostructureRequest, MicrostructureResponse, ReadyRequest as GrpcReadyRequest,
    ReadyResponse as GrpcReadyResponse, SlippageRequest, SlippageResponse, SubmitOrderRequest,
    SubmitOrderResponse,
};
use tonic::{Request, Response, Status};
use std::time::Instant;

use crate::api::metrics::{record_request, record_response};

#[derive(Default)]
pub struct ExecutionServiceImpl;

#[tonic::async_trait]
impl ExecutionService for ExecutionServiceImpl {
    async fn evaluate_execution(
        &self,
        _req: Request<EvaluateExecutionRequest>,
    ) -> Result<Response<EvaluateExecutionResponse>, Status> {
        let start = Instant::now();
        record_request("EvaluateExecution");

        // Adapter pattern: No business logic here. Just a mock response for now, 
        // to be wired to the real engine in the next phase.
        let response = EvaluateExecutionResponse {
            executable: true,
            estimated_slippage: "0.01".to_string(),
            probability: "0.95".to_string(),
        };

        record_response("EvaluateExecution", "ok", start.elapsed());
        Ok(Response::new(response))
    }

    async fn submit_order(
        &self,
        req: Request<SubmitOrderRequest>,
    ) -> Result<Response<SubmitOrderResponse>, Status> {
        let start = Instant::now();
        record_request("SubmitOrder");

        let response = SubmitOrderResponse {
            request_id: req.into_inner().request_id,
            result: None,
            order_id: "order-123".to_string(),
            state: 0,
            submitted_at: None,
            rejection_reason: "".to_string(),
            error_details: None,
        };

        record_response("SubmitOrder", "ok", start.elapsed());
        Ok(Response::new(response))
    }

    async fn get_order_state(
        &self,
        _req: Request<GetOrderStateRequest>,
    ) -> Result<Response<GetOrderStateResponse>, Status> {
        let start = Instant::now();
        record_request("GetOrderState");

        let response = GetOrderStateResponse { order: None };

        record_response("GetOrderState", "ok", start.elapsed());
        Ok(Response::new(response))
    }

    async fn get_position_state(
        &self,
        req: Request<GetPositionStateRequest>,
    ) -> Result<Response<GetPositionStateResponse>, Status> {
        let start = Instant::now();
        record_request("GetPositionState");

        let response = GetPositionStateResponse {
            position_id: req.into_inner().position_id,
            current_volume: "0.0".to_string(),
            unrealized_pnl: "0.0".to_string(),
        };

        record_response("GetPositionState", "ok", start.elapsed());
        Ok(Response::new(response))
    }

    async fn get_execution_risk(
        &self,
        _req: Request<ExecutionRiskRequest>,
    ) -> Result<Response<ExecutionRiskResponse>, Status> {
        let start = Instant::now();
        record_request("GetExecutionRisk");

        let response = ExecutionRiskResponse {
            risk_acceptable: true,
            risk_score: "0.0".to_string(),
            margin_required: "0.0".to_string(),
        };

        record_response("GetExecutionRisk", "ok", start.elapsed());
        Ok(Response::new(response))
    }

    async fn get_liquidity_profile(
        &self,
        _req: Request<LiquidityProfileRequest>,
    ) -> Result<Response<LiquidityProfileResponse>, Status> {
        let start = Instant::now();
        record_request("GetLiquidityProfile");

        let response = LiquidityProfileResponse {
            bid_liquidity: "0.0".to_string(),
            ask_liquidity: "0.0".to_string(),
            depth_score: "0.0".to_string(),
        };

        record_response("GetLiquidityProfile", "ok", start.elapsed());
        Ok(Response::new(response))
    }

    async fn get_slippage_metrics(
        &self,
        _req: Request<SlippageRequest>,
    ) -> Result<Response<SlippageResponse>, Status> {
        let start = Instant::now();
        record_request("GetSlippageMetrics");

        let response = SlippageResponse {
            slippage_amount: "0.0".to_string(),
            slippage_bps: "0.0".to_string(),
        };

        record_response("GetSlippageMetrics", "ok", start.elapsed());
        Ok(Response::new(response))
    }

    async fn get_latency_metrics(
        &self,
        _req: Request<LatencyRequest>,
    ) -> Result<Response<LatencyResponse>, Status> {
        let start = Instant::now();
        record_request("GetLatencyMetrics");

        let response = LatencyResponse {
            network_latency_ms: "0.0".to_string(),
            processing_latency_ms: "0.0".to_string(),
        };

        record_response("GetLatencyMetrics", "ok", start.elapsed());
        Ok(Response::new(response))
    }

    async fn get_microstructure_score(
        &self,
        _req: Request<MicrostructureRequest>,
    ) -> Result<Response<MicrostructureResponse>, Status> {
        let start = Instant::now();
        record_request("GetMicrostructureScore");

        let response = MicrostructureResponse {
            tick_volatility: "0.0".to_string(),
            order_book_imbalance: "0.0".to_string(),
        };

        record_response("GetMicrostructureScore", "ok", start.elapsed());
        Ok(Response::new(response))
    }

    async fn health(
        &self,
        _req: Request<GrpcHealthRequest>,
    ) -> Result<Response<GrpcHealthResponse>, Status> {
        let start = Instant::now();
        record_request("Health");

        let response = GrpcHealthResponse {
            status: "alive".to_string(),
        };

        record_response("Health", "ok", start.elapsed());
        Ok(Response::new(response))
    }

    async fn ready(
        &self,
        _req: Request<GrpcReadyRequest>,
    ) -> Result<Response<GrpcReadyResponse>, Status> {
        let start = Instant::now();
        record_request("Ready");

        let response = GrpcReadyResponse {
            status: "ready".to_string(),
        };

        record_response("Ready", "ok", start.elapsed());
        Ok(Response::new(response))
    }
}
