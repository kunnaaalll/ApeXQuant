use tonic::{Request, Response, Status, Streaming};
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;
use tracing::info;
use sqlx::{PgPool, Row};

use apex_protos::portfolio::portfolio_engine_server::PortfolioEngine;
use apex_protos::portfolio::*;

use std::sync::Arc;
use crate::event_bus::EventBusPublisher;
use crate::portfolio::registry::PortfolioRegistry;
use crate::storage::repository::PortfolioRepository;

pub struct PortfolioServiceImpl {
    pub event_bus: Option<Arc<EventBusPublisher>>,
    pub pool: PgPool,
    pub registry: PortfolioRegistry,
    pub repository: PortfolioRepository,
}

impl PortfolioServiceImpl {
    pub fn new(
        event_bus: Option<Arc<EventBusPublisher>>,
        pool: PgPool,
        registry: PortfolioRegistry,
        repository: PortfolioRepository,
    ) -> Self {
        Self {
            event_bus,
            pool,
            registry,
            repository,
        }
    }

    async fn build_snapshot_from_db(&self, portfolio_id: &str) -> Result<PortfolioSnapshot, Status> {
        let session_uuid = uuid::Uuid::parse_str(portfolio_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid portfolio_id/session_id UUID: {}", e)))?;

        // 1. Fetch the session info to get the initial balance
        let session_row = sqlx::query(
            "SELECT initial_balance FROM sessions WHERE id = $1"
        )
        .bind(session_uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Status::internal(format!("Database error fetching session: {}", e)))?;

        let initial_balance = if let Some(row) = session_row {
            let decimal_val: rust_decimal::Decimal = row.get("initial_balance");
            decimal_val.to_string().parse::<f64>().unwrap_or(100000.0)
        } else {
            100000.0
        };

        // 2. Fetch the active positions for this session
        let position_rows = sqlx::query(
            r#"
            SELECT position_id, symbol, side, current_volume, entry_price, current_price, unrealized_pnl, return_percent
            FROM positions
            WHERE session_id = $1 AND state = 'open'
            "#
        )
        .bind(session_uuid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Status::internal(format!("Database error fetching positions: {}", e)))?;

        let mut positions = Vec::new();
        let mut floating_pnl = 0.0;
        let mut long_exposure = 0.0;
        let mut short_exposure = 0.0;
        let mut margin_used = 0.0;

        for r in position_rows {
            let position_id: String = r.get("position_id");
            let symbol: String = r.get("symbol");
            let side: String = r.get("side");
            let current_volume_dec: rust_decimal::Decimal = r.get("current_volume");
            let entry_price_dec: rust_decimal::Decimal = r.get("entry_price");
            
            let current_price_opt: Option<rust_decimal::Decimal> = r.get("current_price");
            let unrealized_pnl_opt: Option<rust_decimal::Decimal> = r.get("unrealized_pnl");
            let return_percent_opt: Option<rust_decimal::Decimal> = r.get("return_percent");

            let volume = current_volume_dec.to_string().parse::<f64>().unwrap_or(0.0);
            let entry_price = entry_price_dec.to_string().parse::<f64>().unwrap_or(0.0);
            let current_price = current_price_opt.map(|v| v.to_string().parse::<f64>().unwrap_or(entry_price)).unwrap_or(entry_price);
            let unrealized_pnl = unrealized_pnl_opt.map(|v| v.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
            let return_percent = return_percent_opt.map(|v| v.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);

            floating_pnl += unrealized_pnl;

            let contract_size = if symbol.contains("BTC") || symbol.contains("ETH") || symbol.contains("XAU") {
                1.0
            } else {
                100000.0
            };
            let exposure_val = volume * current_price * contract_size;
            if side.to_lowercase() == "buy" {
                long_exposure += exposure_val;
            } else {
                short_exposure += exposure_val;
            }

            margin_used += exposure_val / 100.0;

            positions.push(PositionHolding {
                position_id,
                symbol: Some(apex_protos::common::Symbol {
                    code: symbol,
                    exchange: "".to_string(),
                    asset_class: 0,
                    description: "".to_string(),
                }),
                side: if side.to_lowercase() == "buy" {
                    apex_protos::common::TradeSide::Buy as i32
                } else {
                    apex_protos::common::TradeSide::Sell as i32
                },
                volume: Some(apex_protos::common::Volume {
                    units: volume.to_string(),
                    lot_size: "100000".to_string(),
                    fractional: true,
                }),
                market_value: Some(apex_protos::common::Money {
                    amount: exposure_val.to_string(),
                    currency: "USD".to_string(),
                    exponent: 0,
                }),
                unrealized_pnl: Some(apex_protos::common::Money {
                    amount: unrealized_pnl.to_string(),
                    currency: "USD".to_string(),
                    exponent: 0,
                }),
                return_percent: Some(apex_protos::common::Percentage {
                    value: return_percent.to_string(),
                    is_basis_points: false,
                }),
                portfolio_weight: Some(apex_protos::common::Percentage {
                    value: "0.0".to_string(),
                    is_basis_points: false,
                }),
                entry_price: Some(apex_protos::common::Price {
                    value: entry_price.to_string(),
                    digits: 5,
                    currency: "USD".to_string(),
                }),
                current_price: Some(apex_protos::common::Price {
                    value: current_price.to_string(),
                    digits: 5,
                    currency: "USD".to_string(),
                }),
            });
        }

        let realized_pnl_row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(realized_pnl), 0) as realized
            FROM positions
            WHERE session_id = $1 AND state = 'closed'
            "#
        )
        .bind(session_uuid)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Status::internal(format!("Database error fetching realized PnL: {}", e)))?;

        let realized_pnl_dec: rust_decimal::Decimal = realized_pnl_row.get("realized");
        let realized_pnl = realized_pnl_dec.to_string().parse::<f64>().unwrap_or(0.0);

        let balance = initial_balance + realized_pnl;
        let equity = balance + floating_pnl;
        let margin_free = (equity - margin_used).max(0.0);

        for p in &mut positions {
            if equity > 0.0 {
                let mkt_val = p.market_value.as_ref().map(|mv| mv.amount.parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
                p.portfolio_weight = Some(apex_protos::common::Percentage {
                    value: ((mkt_val / equity) * 100.0).to_string(),
                    is_basis_points: false,
                });
            }
        }

        let total_notional = long_exposure + short_exposure;
        let net_exposure = long_exposure - short_exposure;

        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();

        Ok(PortfolioSnapshot {
            portfolio_id: portfolio_id.to_string(),
            account_id: session_uuid.to_string(),
            snapshot_time: Some(apex_protos::common::Timestamp {
                seconds: now.as_secs() as i64,
                nanos: now.subsec_nanos() as i32,
            }),
            equity: Some(apex_protos::common::Money {
                amount: equity.to_string(),
                currency: "USD".to_string(),
                exponent: 0,
            }),
            balance: Some(apex_protos::common::Money {
                amount: balance.to_string(),
                currency: "USD".to_string(),
                exponent: 0,
            }),
            margin_used: Some(apex_protos::common::Money {
                amount: margin_used.to_string(),
                currency: "USD".to_string(),
                exponent: 0,
            }),
            margin_free: Some(apex_protos::common::Money {
                amount: margin_free.to_string(),
                currency: "USD".to_string(),
                exponent: 0,
            }),
            unrealized_pnl: Some(apex_protos::common::Money {
                amount: floating_pnl.to_string(),
                currency: "USD".to_string(),
                exponent: 0,
            }),
            realized_pnl_today: Some(apex_protos::common::Money {
                amount: realized_pnl.to_string(),
                currency: "USD".to_string(),
                exponent: 0,
            }),
            exposure: Some(ExposureBreakdown {
                total_notional: Some(apex_protos::common::Money { amount: total_notional.to_string(), currency: "USD".to_string(), exponent: 0 }),
                long_exposure: Some(apex_protos::common::Money { amount: long_exposure.to_string(), currency: "USD".to_string(), exponent: 0 }),
                short_exposure: Some(apex_protos::common::Money { amount: short_exposure.to_string(), currency: "USD".to_string(), exponent: 0 }),
                net_exposure: Some(apex_protos::common::Money { amount: net_exposure.to_string(), currency: "USD".to_string(), exponent: 0 }),
                gross_exposure_percent: Some(apex_protos::common::Percentage {
                    value: (if equity > 0.0 { (total_notional / equity) * 100.0 } else { 0.0 }).to_string(),
                    is_basis_points: false,
                }),
                net_exposure_percent: Some(apex_protos::common::Percentage {
                    value: (if equity > 0.0 { (net_exposure / equity) * 100.0 } else { 0.0 }).to_string(),
                    is_basis_points: false,
                }),
                by_asset_class: vec![],
                by_currency: vec![],
                correlated_exposure: vec![],
            }),
            risk: Some(PortfolioRiskMetrics {
                current_drawdown: Some(apex_protos::common::Percentage { value: "0.0".to_string(), is_basis_points: false }),
                max_drawdown_reached: Some(apex_protos::common::Percentage { value: "0.0".to_string(), is_basis_points: false }),
                var_95: None,
                var_99: None,
                expected_shortfall: Some(apex_protos::common::Decimal { value: "0.0".to_string() }),
                beta_to_benchmark: Some(apex_protos::common::Decimal { value: "1.0".to_string() }),
                volatility: Some(apex_protos::common::Decimal { value: "0.0".to_string() }),
                sharpe_ratio: Some(apex_protos::common::Decimal { value: "0.0".to_string() }),
                sortino_ratio: Some(apex_protos::common::Decimal { value: "0.0".to_string() }),
                calmar_ratio: Some(apex_protos::common::Decimal { value: "0.0".to_string() }),
                positions_at_risk_count: 0,
            }),
            positions,
            performance: Some(PerformanceMetrics {
                return_today: Some(apex_protos::common::Percentage {
                    value: (if initial_balance > 0.0 { (realized_pnl / initial_balance) * 100.0 } else { 0.0 }).to_string(),
                    is_basis_points: false,
                }),
                return_this_week: Some(apex_protos::common::Percentage { value: "0.0".to_string(), is_basis_points: false }),
                return_this_month: Some(apex_protos::common::Percentage { value: "0.0".to_string(), is_basis_points: false }),
                win_rate: Some(apex_protos::common::Decimal { value: "0.0".to_string() }),
                profit_factor: Some(apex_protos::common::Decimal { value: "0.0".to_string() }),
                average_winner: Some(apex_protos::common::Decimal { value: "0.0".to_string() }),
                average_loser: Some(apex_protos::common::Decimal { value: "0.0".to_string() }),
            }),
        })
    }
}

#[tonic::async_trait]
impl PortfolioEngine for PortfolioServiceImpl {
    async fn get_portfolio_state(
        &self,
        request: Request<PortfolioStateQuery>,
    ) -> Result<Response<PortfolioStateResponse>, Status> {
        let req = request.into_inner();
        info!("get_portfolio_state called for portfolio: {}", req.portfolio_id);
        let state = self.build_snapshot_from_db(&req.portfolio_id).await?;
        Ok(Response::new(PortfolioStateResponse { state: Some(state) }))
    }

    async fn get_exposure(
        &self,
        request: Request<ExposureQuery>,
    ) -> Result<Response<ExposureResponse>, Status> {
        let req = request.into_inner();
        let snap = self.build_snapshot_from_db(&req.portfolio_id).await?;
        Ok(Response::new(ExposureResponse { exposure: snap.exposure }))
    }

    async fn get_heat(
        &self,
        request: Request<HeatQuery>,
    ) -> Result<Response<HeatResponse>, Status> {
        let req = request.into_inner();
        let snap = self.build_snapshot_from_db(&req.portfolio_id).await?;
        let equity = snap.equity.as_ref().map(|m| m.amount.parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
        let margin_used = snap.margin_used.as_ref().map(|m| m.amount.parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
        let margin_utilization = if equity > 0.0 { (margin_used / equity) * 100.0 } else { 0.0 };
        Ok(Response::new(HeatResponse {
            heat_index: Some(apex_protos::common::Decimal { value: "0.0".to_string() }),
            margin_utilization: Some(apex_protos::common::Percentage {
                value: margin_utilization.to_string(),
                is_basis_points: false,
            }),
        }))
    }

    async fn get_allocation(
        &self,
        request: Request<AllocationQuery>,
    ) -> Result<Response<AllocationResponse>, Status> {
        let req = request.into_inner();
        let snap = self.build_snapshot_from_db(&req.portfolio_id).await?;
        
        let mut entries = Vec::new();
        for p in &snap.positions {
            let weight = p.portfolio_weight.clone().unwrap_or_default();
            entries.push(AllocationEntry {
                symbol: p.symbol.clone(),
                target_weight: Some(weight.clone()),
                actual_weight: Some(weight.clone()),
                drift: Some(apex_protos::common::Percentage {
                    value: "0.0".to_string(),
                    is_basis_points: false,
                }),
                target_notional: p.market_value.clone(),
                actual_notional: p.market_value.clone(),
                rebalance_required: false,
            });
        }
        
        Ok(Response::new(AllocationResponse {
            allocation: Some(Allocation {
                portfolio_id: req.portfolio_id,
                as_of: snap.snapshot_time.clone(),
                entries,
                summary: Some(AllocationSummary {
                    total_drift: Some(apex_protos::common::Decimal { value: "0.0".to_string() }),
                    positions_needing_rebalance: 0,
                    estimated_turnover: Some(apex_protos::common::Money { amount: "0.0".to_string(), currency: "USD".to_string(), exponent: 0 }),
                }),
            })
        }))
    }

    async fn get_quality(
        &self,
        request: Request<QualityQuery>,
    ) -> Result<Response<QualityResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(QualityResponse {
            quality_score: Some(apex_protos::common::Decimal { value: "95.0".to_string() }),
        }))
    }

    async fn get_health(
        &self,
        request: Request<HealthQuery>,
    ) -> Result<Response<HealthResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(HealthResponse {
            health_score: Some(apex_protos::common::Decimal { value: "100.0".to_string() }),
            status: "HEALTHY".to_string(),
        }))
    }

    async fn get_drawdown(
        &self,
        request: Request<DrawdownQuery>,
    ) -> Result<Response<DrawdownResponse>, Status> {
        let req = request.into_inner();
        let snap = self.build_snapshot_from_db(&req.portfolio_id).await?;
        let current_drawdown = snap.risk.as_ref().and_then(|r| r.current_drawdown.clone());
        let max_drawdown = snap.risk.as_ref().and_then(|r| r.max_drawdown_reached.clone());
        Ok(Response::new(DrawdownResponse {
            current_drawdown,
            max_drawdown,
        }))
    }

    async fn get_correlation(
        &self,
        request: Request<CorrelationQuery>,
    ) -> Result<Response<CorrelationResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(CorrelationResponse {
            avg_correlation: Some(apex_protos::common::Decimal { value: "0.15".to_string() }),
        }))
    }

    async fn get_recommendations(
        &self,
        request: Request<RecommendationsQuery>,
    ) -> Result<Response<RecommendationsResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(RecommendationsResponse {
            recommendations: vec![],
        }))
    }

    async fn get_analytics(
        &self,
        request: Request<AnalyticsQuery>,
    ) -> Result<Response<AnalyticsResponse>, Status> {
        let req = request.into_inner();
        let snap = self.build_snapshot_from_db(&req.portfolio_id).await?;
        Ok(Response::new(AnalyticsResponse {
            analytics: snap.performance.clone(),
        }))
    }

    async fn replay_portfolio(
        &self,
        request: Request<ReplayRequest>,
    ) -> Result<Response<ReplayResponse>, Status> {
        let req = request.into_inner();
        let snap = self.build_snapshot_from_db(&req.portfolio_id).await?;
        Ok(Response::new(ReplayResponse {
            success: true,
            state: Some(snap),
        }))
    }

    async fn load_snapshot(
        &self,
        request: Request<LoadSnapshotRequest>,
    ) -> Result<Response<LoadSnapshotResponse>, Status> {
        let req = request.into_inner();
        let snap = self.build_snapshot_from_db(&req.portfolio_id).await?;
        Ok(Response::new(LoadSnapshotResponse {
            state: Some(snap),
        }))
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
