use apex_protos::signal::signal_engine_server::SignalEngine as GrpcSignalEngine;
use apex_protos::signal::{
    DetectRequest, DetectResponse, EngineConfiguration, EngineStatus,
    HealthStatus as ProtoHealthStatus, MarketDataChunk, Signal, SignalOutput,
    SubscribeSignalsRequest,
};
use apex_protos::common::{Result as CommonResult, Empty};
use tonic::{Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;
use std::sync::Arc;
use crate::SignalEngine;

pub struct SignalEngineServiceImpl {
    pub engine: Arc<SignalEngine>,
}

impl SignalEngineServiceImpl {
    pub fn new(engine: Arc<SignalEngine>) -> Self {
        Self { engine }
    }
}

#[tonic::async_trait]
impl GrpcSignalEngine for SignalEngineServiceImpl {
    type StreamMarketDataStream = ReceiverStream<Result<SignalOutput, Status>>;

    async fn stream_market_data(
        &self,
        _request: Request<tonic::Streaming<MarketDataChunk>>,
    ) -> Result<Response<Self::StreamMarketDataStream>, Status> {
        let (_tx, rx) = mpsc::channel(1);
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn detect_signals(
        &self,
        _request: Request<DetectRequest>,
    ) -> Result<Response<DetectResponse>, Status> {
        Ok(Response::new(DetectResponse::default()))
    }

    type SubscribeSignalsStream = ReceiverStream<Result<Signal, Status>>;

    async fn subscribe_signals(
        &self,
        _request: Request<SubscribeSignalsRequest>,
    ) -> Result<Response<Self::SubscribeSignalsStream>, Status> {
        let (_tx, rx) = mpsc::channel(1);
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn get_status(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<EngineStatus>, Status> {
        Ok(Response::new(EngineStatus::default()))
    }

    async fn configure(
        &self,
        _request: Request<EngineConfiguration>,
    ) -> Result<Response<CommonResult>, Status> {
        Ok(Response::new(CommonResult { ok: true, error: None }))
    }

    async fn health(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<ProtoHealthStatus>, Status> {
        Ok(Response::new(ProtoHealthStatus::default()))
    }
}
