use tonic::{Request, Response, Status, Streaming};
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;
use tracing::info;

use apex_protos::portfolio::portfolio_engine_server::PortfolioEngine;
use apex_protos::portfolio::*;


use std::sync::Arc;
use crate::event_bus::EventBusPublisher;

pub struct PortfolioServiceImpl {
    pub event_bus: Option<Arc<EventBusPublisher>>,
}

impl PortfolioServiceImpl {
    pub fn new(event_bus: Option<Arc<EventBusPublisher>>) -> Self {
        Self { event_bus }
    }
}

#[tonic::async_trait]
impl PortfolioEngine for PortfolioServiceImpl {
    async fn get_portfolio_state(
        &self,
        request: Request<PortfolioStateQuery>,
    ) -> Result<Response<PortfolioStateResponse>, Status> {
        let _req = request.into_inner();
        info!("get_portfolio_state called");
        Ok(Response::new(PortfolioStateResponse::default()))
    }

    async fn get_exposure(
        &self,
        request: Request<ExposureQuery>,
    ) -> Result<Response<ExposureResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(ExposureResponse::default()))
    }

    async fn get_heat(
        &self,
        request: Request<HeatQuery>,
    ) -> Result<Response<HeatResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(HeatResponse::default()))
    }

    async fn get_allocation(
        &self,
        request: Request<AllocationQuery>,
    ) -> Result<Response<AllocationResponse>, Status> {
        let _req = request.into_inner();
        
        // Publish PortfolioRebalancedEvent if event_bus is configured
        if let Some(bus) = &self.event_bus {
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
            
            let rebalanced_event = apex_protos::events::PortfolioRebalancedEvent {
                portfolio_id: _req.portfolio_id.clone(),
                rebalance_time: Some(apex_protos::common::Timestamp {
                    seconds: now.as_secs() as i64,
                    nanos: now.subsec_nanos() as i32,
                }),
                allocations: vec![],
                total_value: Some(apex_protos::common::Money {
                    amount: "100000.0".to_string(),
                    currency: "USD".to_string(),
                    exponent: 0,
                }),
            };
            
            let event = apex_protos::events::Event {
                event_id: Some(apex_protos::common::Uuid { value: uuid::Uuid::new_v4().as_bytes().to_vec() }),
                spec_version: None,
                occurred_at: Some(apex_protos::common::Timestamp { seconds: now.as_secs() as i64, nanos: now.subsec_nanos() as i32 }),
                published_at: Some(apex_protos::common::Timestamp { seconds: now.as_secs() as i64, nanos: now.subsec_nanos() as i32 }),
                event_type: "PortfolioRebalancedEvent".to_string(),
                source_service: "portfolio-engine".to_string(),
                topic: "portfolio.rebalance".to_string(),
                correlation: None,
                causation_id: "".to_string(),
                deduplication_key: "".to_string(),
                payload: Some(apex_protos::events::event::Payload::PortfolioRebalanced(rebalanced_event)),
                payload_hash: vec![],
            };
            
            if let Err(e) = bus.publish(event).await {
                tracing::warn!("Failed to publish PortfolioRebalancedEvent: {}", e);
            }
        }
        
        Ok(Response::new(AllocationResponse::default()))
    }

    async fn get_quality(
        &self,
        request: Request<QualityQuery>,
    ) -> Result<Response<QualityResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(QualityResponse::default()))
    }

    async fn get_health(
        &self,
        request: Request<HealthQuery>,
    ) -> Result<Response<HealthResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(HealthResponse::default()))
    }

    async fn get_drawdown(
        &self,
        request: Request<DrawdownQuery>,
    ) -> Result<Response<DrawdownResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(DrawdownResponse::default()))
    }

    async fn get_correlation(
        &self,
        request: Request<CorrelationQuery>,
    ) -> Result<Response<CorrelationResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(CorrelationResponse::default()))
    }

    async fn get_recommendations(
        &self,
        request: Request<RecommendationsQuery>,
    ) -> Result<Response<RecommendationsResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(RecommendationsResponse::default()))
    }

    async fn get_analytics(
        &self,
        request: Request<AnalyticsQuery>,
    ) -> Result<Response<AnalyticsResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(AnalyticsResponse::default()))
    }

    async fn replay_portfolio(
        &self,
        request: Request<ReplayRequest>,
    ) -> Result<Response<ReplayResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(ReplayResponse::default()))
    }

    async fn load_snapshot(
        &self,
        request: Request<LoadSnapshotRequest>,
    ) -> Result<Response<LoadSnapshotResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(LoadSnapshotResponse::default()))
    }

    type LoadEventsStream = ReceiverStream<Result<PortfolioEvent, Status>>;

    async fn load_events(
        &self,
        request: Request<LoadEventsRequest>,
    ) -> Result<Response<Self::LoadEventsStream>, Status> {
        let _req = request.into_inner();
        let (_tx, rx) = mpsc::channel(100);
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn get_equity_curve(
        &self,
        request: Request<EquityCurveQuery>,
    ) -> Result<Response<EquityCurveResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(EquityCurveResponse::default()))
    }

    async fn get_symbol_performance(
        &self,
        request: Request<SymbolPerformanceQuery>,
    ) -> Result<Response<SymbolPerformanceResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(SymbolPerformanceResponse::default()))
    }

    async fn get_regime_performance(
        &self,
        request: Request<RegimePerformanceQuery>,
    ) -> Result<Response<RegimePerformanceResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(RegimePerformanceResponse::default()))
    }

    type StreamUpdatesStream = ReceiverStream<Result<PortfolioUpdate, Status>>;

    async fn stream_updates(
        &self,
        request: Request<Streaming<ClientStreamRequest>>,
    ) -> Result<Response<Self::StreamUpdatesStream>, Status> {
        let mut _stream = request.into_inner();
        let (_tx, rx) = mpsc::channel(100);
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type SubscribeEventsStream = ReceiverStream<Result<PortfolioEvent, Status>>;

    async fn subscribe_events(
        &self,
        request: Request<EventSubscriptionRequest>,
    ) -> Result<Response<Self::SubscribeEventsStream>, Status> {
        let _req = request.into_inner();
        let (_tx, rx) = mpsc::channel(100);
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type SubscribeSnapshotsStream = ReceiverStream<Result<PortfolioSnapshot, Status>>;

    async fn subscribe_snapshots(
        &self,
        request: Request<SnapshotSubscriptionRequest>,
    ) -> Result<Response<Self::SubscribeSnapshotsStream>, Status> {
        let _req = request.into_inner();
        let (_tx, rx) = mpsc::channel(100);
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type SubscribeMetricsStream = ReceiverStream<Result<PortfolioMetricsUpdate, Status>>;

    async fn subscribe_metrics(
        &self,
        request: Request<MetricsSubscriptionRequest>,
    ) -> Result<Response<Self::SubscribeMetricsStream>, Status> {
        let _req = request.into_inner();
        let (_tx, rx) = mpsc::channel(100);
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
