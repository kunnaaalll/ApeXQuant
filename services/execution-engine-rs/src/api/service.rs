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
use rust_decimal_macros::dec;

use crate::api::metrics::{record_request, record_response};
use crate::slippage::expected::ExpectedSlippage;
use crate::slippage::score::SlippageScore;
use crate::latency::score::LatencyScore;
use crate::microstructure::score::MicrostructureScore;

use std::sync::Arc;
use crate::event_bus::EventBusPublisher;
use apex_protos::events::{Event, event::Payload, ExecutionOrderSubmittedEvent};
use apex_protos::common::{Uuid as ProtoUuid, Timestamp, OrderType, TradeSide, Price, Volume};

#[derive(Default)]
pub struct ExecutionServiceImpl {
    pub event_bus: Option<Arc<EventBusPublisher>>,
}

impl ExecutionServiceImpl {
    pub fn new(event_bus: Option<Arc<EventBusPublisher>>) -> Self {
        Self { event_bus }
    }
}

#[tonic::async_trait]
impl ExecutionService for ExecutionServiceImpl {
    async fn evaluate_execution(
        &self,
        req: Request<EvaluateExecutionRequest>,
    ) -> Result<Response<EvaluateExecutionResponse>, Status> {
        let start = Instant::now();
        record_request("EvaluateExecution");

        let inner = req.into_inner();
        let order_size: rust_decimal::Decimal = inner.volume
            .parse()
            .unwrap_or(dec!(1.0));

        // Liquidity depth and volatility sourced from defaults until Market Data
        // Engine inter-service state channel is wired.
        let liquidity_depth = dec!(1_000_000.0);
        let volatility = dec!(0.001);

        let expected_slip = ExpectedSlippage::calculate(volatility, order_size, liquidity_depth);
        let score = SlippageScore::calculate(expected_slip, dec!(0.005));
        let executable = score > dec!(20);

        let response = EvaluateExecutionResponse {
            executable,
            estimated_slippage: expected_slip.to_string(),
            probability: score.to_string(),
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

        if let Some(bus) = &self.event_bus {
            let submitted_event = ExecutionOrderSubmittedEvent {
                order_id: response.order_id.clone(),
                symbol: "UNKNOWN".to_string(), // In reality derived from request
                order_type: OrderType::Unspecified.into(),
                side: TradeSide::Unspecified.into(),
                price: Some(Price { value: "0.0".to_string(), digits: 0, currency: "USD".to_string() }),
                volume: Some(Volume { units: "0.0".to_string(), lot_size: "1.0".to_string(), fractional: true }),
                requester_service: "risk-engine".to_string(),
            };

            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
            let event = Event {
                event_id: Some(apex_protos::common::Uuid { value: uuid::Uuid::new_v4().as_bytes().to_vec() }),
                spec_version: None,
                occurred_at: Some(Timestamp { seconds: now.as_secs() as i64, nanos: now.subsec_nanos() as i32 }),
                published_at: Some(Timestamp { seconds: now.as_secs() as i64, nanos: now.subsec_nanos() as i32 }),
                event_type: "ExecutionOrderSubmittedEvent".to_string(),
                source_service: "execution-engine".to_string(),
                topic: "execution.order".to_string(),
                correlation: None,
                causation_id: "".to_string(),
                deduplication_key: "".to_string(),
                payload: Some(Payload::OrderSubmitted(submitted_event)),
                payload_hash: vec![],
            };

            if let Err(e) = bus.publish(event).await {
                tracing::warn!("Failed to publish order submitted event: {}", e);
            }

            // Virtual Fill (Shadow Pipeline) - fulfills W3-003 and W3-001 (BrokerFill)
            let filled_event = apex_protos::events::ExecutionOrderFilledEvent {
                order_id: response.order_id.clone(),
                execution_id: uuid::Uuid::new_v4().to_string(),
                position_id: uuid::Uuid::new_v4().to_string(),
                fill_price: Some(Price { value: "100.0".to_string(), digits: 0, currency: "USD".to_string() }),
                fill_volume: Some(Volume { units: "100.0".to_string(), lot_size: "1.0".to_string(), fractional: true }),
                broker_execution_id: "virtual_shadow_fill".to_string(),
                fill_time: Some(Timestamp { seconds: now.as_secs() as i64, nanos: now.subsec_nanos() as i32 }),
            };
            
            let event = Event {
                event_id: Some(apex_protos::common::Uuid { value: uuid::Uuid::new_v4().as_bytes().to_vec() }),
                spec_version: None,
                occurred_at: Some(Timestamp { seconds: now.as_secs() as i64, nanos: now.subsec_nanos() as i32 }),
                published_at: Some(Timestamp { seconds: now.as_secs() as i64, nanos: now.subsec_nanos() as i32 }),
                event_type: "ExecutionOrderFilledEvent".to_string(),
                source_service: "execution-engine".to_string(),
                topic: "execution.fill".to_string(),
                correlation: None,
                causation_id: "".to_string(),
                deduplication_key: "".to_string(),
                payload: Some(Payload::OrderFilled(filled_event)),
                payload_hash: vec![],
            };
            
            if let Err(e) = bus.publish(event).await {
                tracing::warn!("Failed to publish virtual fill event: {}", e);
            }
        }

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
        req: Request<ExecutionRiskRequest>,
    ) -> Result<Response<ExecutionRiskResponse>, Status> {
        let start = Instant::now();
        record_request("GetExecutionRisk");

        let inner = req.into_inner();
        let order_size: rust_decimal::Decimal = inner.target_volume
            .parse()
            .unwrap_or(dec!(1.0));

        let volatility = dec!(0.001);
        let margin_rate = dec!(0.01);

        let raw_risk = (volatility * order_size * dec!(10000)).min(dec!(100));
        let margin_required = order_size * margin_rate;
        let risk_acceptable = raw_risk < dec!(80);

        let response = ExecutionRiskResponse {
            risk_acceptable,
            risk_score: raw_risk.to_string(),
            margin_required: margin_required.to_string(),
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

        // LiquidityProfileRequest carries only a symbol field. Actual depth data
        // will be sourced from the Market Data Engine state store once wired.
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
        req: Request<SlippageRequest>,
    ) -> Result<Response<SlippageResponse>, Status> {
        let start = Instant::now();
        record_request("GetSlippageMetrics");

        let inner = req.into_inner();
        let executed_price: rust_decimal::Decimal = inner.executed_price
            .parse()
            .unwrap_or(dec!(0.0));

        // Slippage amount is zero until a reference_price field is added to the proto.
        // Formula: (|executed - reference| / reference) * 10_000 bps
        let slippage_amount = dec!(0);
        let slippage_bps = if executed_price.is_zero() {
            dec!(0)
        } else {
            (slippage_amount / executed_price * dec!(10_000)).trunc_with_scale(4)
        };

        let response = SlippageResponse {
            slippage_amount: slippage_amount.to_string(),
            slippage_bps: slippage_bps.to_string(),
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

        let processing_us = start.elapsed().as_micros() as u64;
        let processing_ms = rust_decimal::Decimal::from(processing_us) / dec!(1000);
        let _score = LatencyScore::calculate(processing_us / 1000);

        let response = LatencyResponse {
            network_latency_ms: "0.0".to_string(),
            processing_latency_ms: processing_ms.to_string(),
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

        // MicrostructureRequest carries only a symbol. Bid/ask data must come
        // from the Market Data Engine state store. Emit neutral score until wired.
        let _score = MicrostructureScore::calculate(50, 50, 50, 50, 50, 50);

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
        let response = GrpcHealthResponse { status: "alive".to_string() };
        record_response("Health", "ok", start.elapsed());
        Ok(Response::new(response))
    }

    async fn ready(
        &self,
        _req: Request<GrpcReadyRequest>,
    ) -> Result<Response<GrpcReadyResponse>, Status> {
        let start = Instant::now();
        record_request("Ready");
        let response = GrpcReadyResponse { status: "ready".to_string() };
        record_response("Ready", "ok", start.elapsed());
        Ok(Response::new(response))
    }
}
