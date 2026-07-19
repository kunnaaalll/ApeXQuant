use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use apex_protos::common::{
    Decimal as ProtoDecimal, Empty, Money, Price, Result as CommonResult, Symbol, Volume,
};
use apex_protos::position::position_engine_server::PositionEngine;
use apex_protos::position::{
    BreakevenRequest, ClosePositionRequest, ClosePositionResponse, ListPositionsRequest,
    ListPositionsResponse, ModifyStopsRequest, ModifyStopsResponse, OpenPositionRequest,
    OpenPositionResponse, PartialCloseRequest, Position as ProtoPosition, PositionActionResponse,
    PositionFilter, PositionQuery, PositionState as ProtoPositionState, PositionUpdate,
    TrailingStopRequest,
};

use crate::positions::{PositionRegistry, PositionState, PositionTracker};
use crate::storage::PostgresStore;

pub struct PositionEngineService {
    registry: PositionRegistry,
    store: Arc<PostgresStore>,
    mt5_bridge_url: String,
    client: reqwest::Client,
}

impl PositionEngineService {
    pub fn new(
        registry: PositionRegistry,
        store: Arc<PostgresStore>,
        mt5_bridge_url: String,
    ) -> Self {
        Self {
            registry,
            store,
            mt5_bridge_url,
            client: reqwest::Client::new(),
        }
    }
}

fn to_proto_decimal(dec: rust_decimal::Decimal) -> Option<ProtoDecimal> {
    Some(ProtoDecimal {
        value: dec.to_string(),
    })
}

fn to_proto_price(dec: Option<rust_decimal::Decimal>) -> Option<Price> {
    dec.map(|d| Price {
        value: d.to_string(),
        digits: 5,
        currency: "USD".to_string(),
    })
}

fn to_proto_volume(dec: rust_decimal::Decimal) -> Option<Volume> {
    Some(Volume {
        units: dec.to_string(),
        lot_size: "1.0".to_string(),
        fractional: true,
    })
}

fn to_proto_money(dec: rust_decimal::Decimal) -> Option<Money> {
    Some(Money {
        amount: dec.to_string(),
        currency: "USD".to_string(),
        exponent: 2,
    })
}

fn to_proto_position(tracker: &PositionTracker) -> ProtoPosition {
    let side_val = if tracker.side.to_lowercase() == "buy" {
        1
    } else {
        2
    }; // BUY = 1, SELL = 2
    let proto_state = match tracker.state {
        PositionState::Opening => ProtoPositionState::Opening,
        PositionState::Active => ProtoPositionState::Open,
        PositionState::ScalingIn => ProtoPositionState::Open,
        PositionState::ScalingOut => ProtoPositionState::PartialClose,
        PositionState::Reducing => ProtoPositionState::Open,
        PositionState::Closing => ProtoPositionState::Closing,
        PositionState::Closed => ProtoPositionState::Closed,
        PositionState::Archived => ProtoPositionState::Closed,
        PositionState::Invalid => ProtoPositionState::Error,
    };

    ProtoPosition {
        position_id: tracker.position_id.to_string(),
        parent_position_id: "".to_string(),
        symbol: Some(Symbol {
            code: tracker.symbol.clone(),
            exchange: "".to_string(),
            asset_class: 1, // ASSET_CLASS_FOREX
            description: "".to_string(),
        }),
        side: side_val,
        initial_volume: to_proto_volume(tracker.initial_size),
        current_volume: to_proto_volume(tracker.current_size),
        closed_volume: to_proto_volume(rust_decimal::Decimal::ZERO),
        entry_price: Some(Price {
            value: tracker.initial_entry_price.to_string(),
            digits: 5,
            currency: "USD".to_string(),
        }),
        current_price: Some(Price {
            value: tracker.current_price.to_string(),
            digits: 5,
            currency: "USD".to_string(),
        }),
        exit_price: None,
        average_close_price: None,
        stop_loss: to_proto_price(tracker.current_stop_loss),
        take_profit: to_proto_price(tracker.initial_take_profit),
        breakeven_price: None,
        trailing_stop_price: to_proto_price(tracker.current_stop_loss),
        unrealized_pnl: to_proto_money(tracker.unrealized_pnl),
        realized_pnl: to_proto_money(tracker.realized_pnl),
        commission_paid: None,
        swap_paid: None,
        return_percent: None,
        pips_from_entry: None,
        stop_distance_pips: None,
        tp_distance_pips: None,
        risk_reward_ratio: None,
        state: proto_state.into(),
        is_breakeven: false,
        has_trailing_stop: tracker.current_stop_loss.is_some(),
        opened_at: None,
        last_modified_at: None,
        closed_at: None,
        signal_id: "".to_string(),
        strategy_id: "".to_string(),
        execution_order_id: "".to_string(),
        timeframe: Some(apex_protos::common::Timeframe {
            value: 15,
            unit: 1, // TIME_UNIT_MINUTE
        }),
        broker_position_id: "".to_string(),
        management: None,
        history: vec![],
    }
}

#[tonic::async_trait]
impl PositionEngine for PositionEngineService {
    async fn open_position(
        &self,
        request: Request<OpenPositionRequest>,
    ) -> std::result::Result<Response<OpenPositionResponse>, Status> {
        let req = request.into_inner();
        let position_id = Uuid::new_v4();

        let sym_str = req
            .symbol
            .map(|s| s.code)
            .unwrap_or_else(|| "EURUSD".to_string());
        let side_str = if req.side == 1 { "buy" } else { "sell" };
        let init_vol = req
            .volume
            .and_then(|v| v.units.parse::<rust_decimal::Decimal>().ok())
            .unwrap_or(rust_decimal::Decimal::ZERO);
        let entry = req
            .entry_price
            .and_then(|p| p.value.parse::<rust_decimal::Decimal>().ok())
            .unwrap_or(rust_decimal::Decimal::ZERO);

        let mut tracker =
            PositionTracker::new(position_id, sym_str, side_str.to_string(), init_vol, entry);
        tracker.state = PositionState::Active;
        tracker.current_stop_loss = req
            .stop_loss
            .and_then(|p| p.value.parse::<rust_decimal::Decimal>().ok());
        tracker.initial_take_profit = req
            .take_profit
            .and_then(|p| p.value.parse::<rust_decimal::Decimal>().ok());

        self.registry.insert(tracker.clone());
        let _ = self.store.save_position(&tracker).await;

        Ok(Response::new(OpenPositionResponse {
            request_id: req.request_id,
            success: true,
            position: Some(to_proto_position(&tracker)),
            execution_order_id: req.execution_order_id,
            opened_at: None,
            error: None,
        }))
    }

    async fn close_position(
        &self,
        request: Request<ClosePositionRequest>,
    ) -> std::result::Result<Response<ClosePositionResponse>, Status> {
        let req = request.into_inner();
        let position_id = Uuid::parse_str(&req.position_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        if let Some(mut tracker) = self.registry.get(&position_id) {
            tracker.state = PositionState::Closed;
            self.registry.insert(tracker.clone());
            let _ = self.store.save_position(&tracker).await;
            self.registry.remove(&position_id);

            Ok(Response::new(ClosePositionResponse {
                success: true,
                position: Some(to_proto_position(&tracker)),
                result: None,
                closed_at: None,
                error: None,
            }))
        } else {
            Err(Status::not_found("Position not found"))
        }
    }

    async fn modify_stops(
        &self,
        request: Request<ModifyStopsRequest>,
    ) -> std::result::Result<Response<ModifyStopsResponse>, Status> {
        let req = request.into_inner();
        let position_id = Uuid::parse_str(&req.position_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        if let Some(mut tracker) = self.registry.get(&position_id) {
            if let Some(new_sl) = req.new_stop_loss {
                if let Ok(sl_dec) = new_sl.value.parse::<rust_decimal::Decimal>() {
                    tracker.current_stop_loss = Some(sl_dec);
                }
            }
            if let Some(new_tp) = req.new_take_profit {
                if let Ok(tp_dec) = new_tp.value.parse::<rust_decimal::Decimal>() {
                    tracker.initial_take_profit = Some(tp_dec);
                }
            }

            self.registry.insert(tracker.clone());
            let _ = self.store.save_position(&tracker).await;

            Ok(Response::new(ModifyStopsResponse {
                success: true,
                position: Some(to_proto_position(&tracker)),
                result: None,
                error: None,
            }))
        } else {
            Err(Status::not_found("Position not found"))
        }
    }

    async fn get_position(
        &self,
        request: Request<PositionQuery>,
    ) -> std::result::Result<Response<ProtoPosition>, Status> {
        let req = request.into_inner();
        let position_id = Uuid::parse_str(&req.position_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        if let Some(tracker) = self.registry.get(&position_id) {
            Ok(Response::new(to_proto_position(&tracker)))
        } else if let Ok(Some(tracker)) = self.store.get_position(position_id).await {
            Ok(Response::new(to_proto_position(&tracker)))
        } else {
            Err(Status::not_found("Position not found"))
        }
    }

    async fn list_positions(
        &self,
        _request: Request<ListPositionsRequest>,
    ) -> std::result::Result<Response<ListPositionsResponse>, Status> {
        // Collect active positions from memory
        let mut proto_positions = Vec::new();
        for entry in self.registry.positions.iter() {
            proto_positions.push(to_proto_position(entry.value()));
        }

        Ok(Response::new(ListPositionsResponse {
            positions: proto_positions,
            summary: None,
            page_info: None,
        }))
    }

    type SubscribePositionUpdatesStream =
        tokio_stream::wrappers::ReceiverStream<std::result::Result<PositionUpdate, Status>>;

    async fn subscribe_position_updates(
        &self,
        _request: Request<PositionFilter>,
    ) -> std::result::Result<Response<Self::SubscribePositionUpdatesStream>, Status> {
        let (_tx, rx) = tokio::sync::mpsc::channel(10);
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }

    async fn set_breakeven(
        &self,
        request: Request<BreakevenRequest>,
    ) -> std::result::Result<Response<PositionActionResponse>, Status> {
        let req = request.into_inner();
        let position_id = Uuid::parse_str(&req.position_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        if let Some(mut tracker) = self.registry.get(&position_id) {
            // Move stop loss to entry price
            tracker.current_stop_loss = Some(tracker.initial_entry_price);
            self.registry.insert(tracker.clone());
            let _ = self.store.save_position(&tracker).await;

            Ok(Response::new(PositionActionResponse {
                success: true,
                position: Some(to_proto_position(&tracker)),
                message: "Stops moved to breakeven successfully".to_string(),
                error: None,
            }))
        } else {
            Err(Status::not_found("Position not found"))
        }
    }

    async fn enable_trailing_stop(
        &self,
        request: Request<TrailingStopRequest>,
    ) -> std::result::Result<Response<PositionActionResponse>, Status> {
        let req = request.into_inner();
        let position_id = Uuid::parse_str(&req.position_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        if let Some(mut tracker) = self.registry.get(&position_id) {
            // Adjust tracker stop configuration (trailing is handled dynamically by background loop)
            // Just initialize a starting trailing stop-loss price if it wasn't set
            if tracker.current_stop_loss.is_none() {
                let dist = req
                    .distance
                    .as_ref()
                    .map(|d| {
                        d.value
                            .parse::<rust_decimal::Decimal>()
                            .unwrap_or(rust_decimal::Decimal::ZERO)
                    })
                    .unwrap_or(rust_decimal::Decimal::ZERO);
                if dist > rust_decimal::Decimal::ZERO {
                    let price = tracker.current_price;
                    tracker.current_stop_loss = Some(if tracker.side == "buy" {
                        price - dist
                    } else {
                        price + dist
                    });
                }
            }

            self.registry.insert(tracker.clone());
            let _ = self.store.save_position(&tracker).await;

            Ok(Response::new(PositionActionResponse {
                success: true,
                position: Some(to_proto_position(&tracker)),
                message: "Trailing stop-loss enabled".to_string(),
                error: None,
            }))
        } else {
            Err(Status::not_found("Position not found"))
        }
    }

    async fn partial_close(
        &self,
        request: Request<PartialCloseRequest>,
    ) -> std::result::Result<Response<ClosePositionResponse>, Status> {
        let req = request.into_inner();
        let position_id = Uuid::parse_str(&req.position_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        if let Some(mut tracker) = self.registry.get(&position_id) {
            // Deduct size
            let close_size = req
                .close_amount
                .and_then(|a| match a {
                    apex_protos::position::partial_close_request::CloseAmount::Volume(v) => {
                        v.units.parse::<rust_decimal::Decimal>().ok()
                    }
                    _ => None,
                })
                .unwrap_or(rust_decimal::Decimal::ZERO);

            if close_size > rust_decimal::Decimal::ZERO && close_size < tracker.current_size {
                tracker.current_size -= close_size;
                tracker.state = PositionState::ScalingOut;
                self.registry.insert(tracker.clone());
                let _ = self.store.save_position(&tracker).await;
            }

            Ok(Response::new(ClosePositionResponse {
                success: true,
                position: Some(to_proto_position(&tracker)),
                result: None,
                closed_at: None,
                error: None,
            }))
        } else {
            Err(Status::not_found("Position not found"))
        }
    }

    async fn health(
        &self,
        _request: Request<Empty>,
    ) -> std::result::Result<Response<CommonResult>, Status> {
        Ok(Response::new(CommonResult {
            ok: true,
            error: None,
        }))
    }
}
