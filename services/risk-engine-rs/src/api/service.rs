use std::sync::Arc;
use tonic::{Request, Response, Status};
use std::str::FromStr;
use rust_decimal::Decimal as RustDecimal;

use apex_protos::common::{Result as GrpcResult, Empty, Money, Percentage, Decimal as ProtoDecimal, Price};
use apex_protos::risk::{
    risk_engine_server::RiskEngine as GrpcRiskEngine, AssessRequest, AssessResponse,
    ExposureFilter, ExposureUpdate, PositionSizeRequest, PositionSizeResponse, RiskConfiguration,
    RiskLimits, TradeOutcome, ValidateOrderRequest, ValidateOrderResponse, Warning, PositionRecommendation
};

use crate::{RiskEngine, RiskInputs, sessions::MarketSession};

pub struct RiskService {
    engine: Arc<RiskEngine>,
}

impl RiskService {
    pub fn new(engine: Arc<RiskEngine>) -> Self {
        Self { engine }
    }
}

// Helpers for conversion
fn parse_money(money: &Option<Money>) -> RustDecimal {
    money.as_ref()
        .and_then(|m| RustDecimal::from_str(&m.amount).ok())
        .unwrap_or(RustDecimal::ZERO)
}

fn parse_price(price: &Option<Price>) -> RustDecimal {
    price.as_ref()
        .and_then(|p| RustDecimal::from_str(&p.value).ok())
        .unwrap_or(RustDecimal::ZERO)
}

#[tonic::async_trait]
impl GrpcRiskEngine for RiskService {
    async fn assess_risk(
        &self,
        request: Request<AssessRequest>,
    ) -> Result<Response<AssessResponse>, Status> {
        let req = request.into_inner();
        
        let context = req.context.unwrap_or_default();
        
        let inputs = RiskInputs {
            equity: parse_money(&context.account_equity),
            balance: parse_money(&context.account_balance),
            symbol: req.symbol.map(|s| s.code).unwrap_or_default(),
            direction: if req.side == 1 { 1 } else { -1 },
            entry_price: parse_price(&req.entry_price),
            stop_loss: parse_price(&req.stop_loss),
            take_profit: None,
            signal_confidence: RustDecimal::from_f64_retain(req.signal_confidence).unwrap_or(RustDecimal::ZERO),
            confluence_score: RustDecimal::new(75, 1),
            regime_quality: RustDecimal::new(7, 1),
            pattern_quality: RustDecimal::new(75, 2),
            atr: None,
            spread: RustDecimal::new(1, 5),
            open_positions: Vec::new(),
            daily_pnl: RustDecimal::ZERO,
            daily_trades: 0,
            recent_trades: Vec::new(),
            session: MarketSession::London,
        };

        match self.engine.assess(&inputs).await {
            Ok(assessment) => {
                let response = AssessResponse {
                    assessment_id: req.assessment_id,
                    approved: assessment.approved,
                    approval_level: if assessment.approved { 1 } else { 4 },
                    tier: 1,
                    checks_performed: vec![],
                    violations: vec![],
                    warnings: assessment.warnings.into_iter().map(|w| Warning {
                        warning_type: "engine_warning".to_string(),
                        description: w,
                        current_value: None,
                        threshold_value: None,
                        severity: 3,
                    }).collect(),
                    recommendation: Some(PositionRecommendation {
                        suggested_lots: Some(ProtoDecimal { value: assessment.lot_size.to_string() }),
                        max_lots: None,
                        risk_amount: Some(Money { amount: assessment.capital_at_risk.to_string(), currency: "USD".to_string(), exponent: 2 }),
                        margin_required: None,
                        risk_percent_of_equity: Some(Percentage { value: assessment.risk_percent.to_string(), is_basis_points: false }),
                        position_sizing_method: "adaptive".to_string(),
                        kelly_fraction: Default::default(),
                    }),
                    assessed_at: None,
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                Err(Status::internal(format!("Risk assessment failed: {}", e)))
            }
        }
    }

    async fn validate_order(
        &self,
        request: Request<ValidateOrderRequest>,
    ) -> Result<Response<ValidateOrderResponse>, Status> {
        let req = request.into_inner();
        let context = req.context.unwrap_or_default();
        let order = req.order.unwrap_or_default();
        
        let inputs = RiskInputs {
            equity: parse_money(&context.account_equity),
            balance: parse_money(&context.account_balance),
            symbol: order.symbol.map(|s| s.code).unwrap_or_default(),
            direction: if order.side == 1 { 1 } else { -1 },
            entry_price: parse_price(&order.entry_price),
            stop_loss: parse_price(&order.stop_loss),
            take_profit: Some(parse_price(&order.take_profit)),
            signal_confidence: RustDecimal::new(80, 2),
            confluence_score: RustDecimal::new(75, 1),
            regime_quality: RustDecimal::new(7, 1),
            pattern_quality: RustDecimal::new(75, 2),
            atr: None,
            spread: RustDecimal::new(1, 5),
            open_positions: Vec::new(),
            daily_pnl: RustDecimal::ZERO,
            daily_trades: 0,
            recent_trades: Vec::new(),
            session: MarketSession::London,
        };

        match self.engine.assess(&inputs).await {
            Ok(assessment) => {
                let response = ValidateOrderResponse {
                    valid: assessment.approved,
                    results: vec![],
                    violations: vec![],
                    warnings: assessment.warnings.into_iter().map(|w| Warning {
                        warning_type: "engine_warning".to_string(),
                        description: w,
                        current_value: None,
                        threshold_value: None,
                        severity: 3,
                    }).collect(),
                    validated_at: None,
                    suggested_modification: None,
                };
                Ok(Response::new(response))
            }
            Err(e) => Err(Status::internal(format!("Validation failed: {}", e))),
        }
    }

    async fn calculate_position_size(
        &self,
        request: Request<PositionSizeRequest>,
    ) -> Result<Response<PositionSizeResponse>, Status> {
        let req = request.into_inner();
        let context = req.context.unwrap_or_default();
        
        let inputs = RiskInputs {
            equity: parse_money(&context.account_equity),
            balance: parse_money(&context.account_balance),
            symbol: req.symbol.map(|s| s.code).unwrap_or_default(),
            direction: 1, // Assume long for size calculation if side not provided
            entry_price: parse_price(&req.entry_price),
            stop_loss: parse_price(&req.stop_loss),
            take_profit: None,
            signal_confidence: RustDecimal::new(80, 2),
            confluence_score: RustDecimal::new(75, 1),
            regime_quality: RustDecimal::new(7, 1),
            pattern_quality: RustDecimal::new(75, 2),
            atr: None,
            spread: RustDecimal::new(1, 5),
            open_positions: Vec::new(),
            daily_pnl: RustDecimal::ZERO,
            daily_trades: 0,
            recent_trades: Vec::new(),
            session: MarketSession::London,
        };

        match self.engine.assess(&inputs).await {
            Ok(assessment) => {
                let response = PositionSizeResponse {
                    recommended_lots: Some(ProtoDecimal { value: assessment.lot_size.to_string() }),
                    risk_amount: Some(Money { amount: assessment.capital_at_risk.to_string(), currency: "USD".to_string(), exponent: 2 }),
                    margin_required: None,
                    risk_percent: Some(Percentage { value: assessment.risk_percent.to_string(), is_basis_points: false }),
                    stop_distance_atr: None,
                    sizing_rationale: "adaptive".to_string(),
                    alternatives: vec![],
                };
                Ok(Response::new(response))
            }
            Err(e) => Err(Status::internal(format!("Position size calculation failed: {}", e))),
        }
    }

    async fn get_limits(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<RiskLimits>, Status> {
        Ok(Response::new(RiskLimits {
            retrieved_at: None,
            mode: "normal".to_string(),
            mode_since: None,
            account_limits: None,
            account_usage: None,
            position_limits: None,
            symbol_limits: vec![],
            daily_limits: None,
            daily_usage: None,
        }))
    }

    async fn configure(
        &self,
        _request: Request<RiskConfiguration>,
    ) -> Result<Response<GrpcResult>, Status> {
        Ok(Response::new(GrpcResult {
            ok: true,
            error: None,
        }))
    }

    type SubscribeExposureStream =
        tonic::codegen::tokio_stream::wrappers::ReceiverStream<Result<ExposureUpdate, Status>>;

    async fn subscribe_exposure(
        &self,
        _request: Request<ExposureFilter>,
    ) -> Result<Response<Self::SubscribeExposureStream>, Status> {
        Err(Status::unimplemented("subscribe_exposure not yet implemented"))
    }

    async fn record_outcome(
        &self,
        _request: Request<TradeOutcome>,
    ) -> Result<Response<GrpcResult>, Status> {
        Err(Status::unimplemented("record_outcome not yet implemented"))
    }

    async fn health(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<GrpcResult>, Status> {
        Ok(Response::new(GrpcResult {
            ok: true,
            error: None,
        }))
    }
}
