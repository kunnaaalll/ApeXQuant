use apex_protos::risk::risk_engine_server::RiskEngine;
use apex_protos::risk::*;
use tonic::{Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;

#[derive(Default)]
pub struct RiskServiceImpl;

#[tonic::async_trait]
impl RiskEngine for RiskServiceImpl {
    async fn get_risk_state(
        &self,
        request: Request<RiskStateQuery>,
    ) -> Result<Response<RiskStateResponse>, Status> {
        let req = request.into_inner();
        let response = RiskStateResponse {
            account_id: req.account_id,
            as_of: Some(apex_protos::common::Timestamp {
                seconds: time::OffsetDateTime::now_utc().unix_timestamp(),
                nanos: time::OffsetDateTime::now_utc().nanosecond() as i32,
            }),
            state: "ACTIVE".to_string(),
        };
        Ok(Response::new(response))
    }

    async fn get_drawdown(
        &self,
        request: Request<DrawdownQuery>,
    ) -> Result<Response<DrawdownResponse>, Status> {
        let req = request.into_inner();
        let response = DrawdownResponse {
            account_id: req.account_id,
            current_drawdown: None,
            max_drawdown: None,
        };
        Ok(Response::new(response))
    }

    async fn get_exposure(
        &self,
        request: Request<ExposureQuery>,
    ) -> Result<Response<ExposureResponse>, Status> {
        let req = request.into_inner();
        let response = ExposureResponse {
            account_id: req.account_id,
            total_exposure: None,
        };
        Ok(Response::new(response))
    }

    async fn get_correlation(
        &self,
        request: Request<CorrelationQuery>,
    ) -> Result<Response<CorrelationResponse>, Status> {
        let req = request.into_inner();
        let response = CorrelationResponse {
            account_id: req.account_id,
            matrix: None,
        };
        Ok(Response::new(response))
    }

    async fn get_hidden_leverage(
        &self,
        request: Request<HiddenLeverageQuery>,
    ) -> Result<Response<HiddenLeverageResponse>, Status> {
        let req = request.into_inner();
        let response = HiddenLeverageResponse {
            account_id: req.account_id,
            leverage_ratio: None,
        };
        Ok(Response::new(response))
    }

    async fn get_historical_var(
        &self,
        request: Request<VarQuery>,
    ) -> Result<Response<VarResponse>, Status> {
        let req = request.into_inner();
        let response = VarResponse {
            account_id: req.account_id,
            value_at_risk: None,
        };
        Ok(Response::new(response))
    }

    async fn get_parametric_var(
        &self,
        request: Request<VarQuery>,
    ) -> Result<Response<VarResponse>, Status> {
        let req = request.into_inner();
        let response = VarResponse {
            account_id: req.account_id,
            value_at_risk: None,
        };
        Ok(Response::new(response))
    }

    async fn get_expected_shortfall(
        &self,
        request: Request<VarQuery>,
    ) -> Result<Response<ExpectedShortfallResponse>, Status> {
        let req = request.into_inner();
        let response = ExpectedShortfallResponse {
            account_id: req.account_id,
            expected_shortfall: None,
        };
        Ok(Response::new(response))
    }

    async fn get_circuit_breaker(
        &self,
        request: Request<CircuitBreakerQuery>,
    ) -> Result<Response<CircuitBreakerResponse>, Status> {
        let req = request.into_inner();
        let response = CircuitBreakerResponse {
            account_id: req.account_id,
            is_tripped: false,
            reason: "".to_string(),
        };
        Ok(Response::new(response))
    }

    async fn get_recommendation(
        &self,
        _request: Request<RecommendationQuery>,
    ) -> Result<Response<RecommendationResponse>, Status> {
        let response = RecommendationResponse {
            recommendation: None,
        };
        Ok(Response::new(response))
    }

    async fn get_stress_assessment(
        &self,
        request: Request<StressQuery>,
    ) -> Result<Response<StressResponse>, Status> {
        let req = request.into_inner();
        let response = StressResponse {
            account_id: req.account_id,
            scenario_id: req.scenario_id,
            estimated_loss: None,
            survived: true,
        };
        Ok(Response::new(response))
    }

    type LoadEventsStream = ReceiverStream<Result<RiskEvent, Status>>;

    async fn load_events(
        &self,
        _request: Request<EventQuery>,
    ) -> Result<Response<Self::LoadEventsStream>, Status> {
        let (_tx, rx) = mpsc::channel(1);
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type SubscribeEventsStream = ReceiverStream<Result<RiskEvent, Status>>;

    async fn subscribe_events(
        &self,
        _request: Request<EventSubscription>,
    ) -> Result<Response<Self::SubscribeEventsStream>, Status> {
        let (_tx, rx) = mpsc::channel(1);
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
