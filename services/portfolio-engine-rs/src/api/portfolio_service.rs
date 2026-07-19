use sqlx::{PgPool, Row};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tracing::info;

use apex_protos::portfolio::portfolio_engine_server::PortfolioEngine;
use apex_protos::portfolio::*;

use crate::event_bus::EventBusPublisher;
use crate::exposure::registry::ExposureRegistry;
use crate::portfolio::registry::PortfolioRegistry;
use crate::storage::repository::PortfolioRepository;
use std::sync::Arc;

pub struct PortfolioServiceImpl {
    pub event_bus: Option<Arc<EventBusPublisher>>,
    pub pool: PgPool,
    pub registry: PortfolioRegistry,
    pub exposure_registry: ExposureRegistry,
    pub repository: PortfolioRepository,
}

impl PortfolioServiceImpl {
    pub fn new(
        event_bus: Option<Arc<EventBusPublisher>>,
        pool: PgPool,
        registry: PortfolioRegistry,
        exposure_registry: ExposureRegistry,
        repository: PortfolioRepository,
    ) -> Self {
        Self {
            event_bus,
            pool,
            registry,
            exposure_registry,
            repository,
        }
    }

    async fn build_snapshot_from_db(
        &self,
        portfolio_id: &str,
    ) -> Result<PortfolioSnapshot, Status> {
        let session_uuid = uuid::Uuid::parse_str(portfolio_id).map_err(|e| {
            Status::invalid_argument(format!("Invalid portfolio_id/session_id UUID: {}", e))
        })?;

        // 1. Fetch the session info to get the initial balance
        let session_row = sqlx::query("SELECT initial_balance FROM sessions WHERE id = $1")
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
            WHERE state = 'open'
            "#
        )
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
            let current_price = current_price_opt
                .map(|v| v.to_string().parse::<f64>().unwrap_or(entry_price))
                .unwrap_or(entry_price);
            let unrealized_pnl = unrealized_pnl_opt
                .map(|v| v.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0);
            let return_percent = return_percent_opt
                .map(|v| v.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0);

            floating_pnl += unrealized_pnl;

            let contract_size =
                if symbol.contains("BTC") || symbol.contains("ETH") || symbol.contains("XAU") {
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
            WHERE state = 'closed'
            "#,
        )
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
                let mkt_val = p
                    .market_value
                    .as_ref()
                    .map(|mv| mv.amount.parse::<f64>().unwrap_or(0.0))
                    .unwrap_or(0.0);
                p.portfolio_weight = Some(apex_protos::common::Percentage {
                    value: ((mkt_val / equity) * 100.0).to_string(),
                    is_basis_points: false,
                });
            }
        }

        let total_notional = long_exposure + short_exposure;
        let net_exposure = long_exposure - short_exposure;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();

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
                total_notional: Some(apex_protos::common::Money {
                    amount: total_notional.to_string(),
                    currency: "USD".to_string(),
                    exponent: 0,
                }),
                long_exposure: Some(apex_protos::common::Money {
                    amount: long_exposure.to_string(),
                    currency: "USD".to_string(),
                    exponent: 0,
                }),
                short_exposure: Some(apex_protos::common::Money {
                    amount: short_exposure.to_string(),
                    currency: "USD".to_string(),
                    exponent: 0,
                }),
                net_exposure: Some(apex_protos::common::Money {
                    amount: net_exposure.to_string(),
                    currency: "USD".to_string(),
                    exponent: 0,
                }),
                gross_exposure_percent: Some(apex_protos::common::Percentage {
                    value: (if equity > 0.0 {
                        (total_notional / equity) * 100.0
                    } else {
                        0.0
                    })
                    .to_string(),
                    is_basis_points: false,
                }),
                net_exposure_percent: Some(apex_protos::common::Percentage {
                    value: (if equity > 0.0 {
                        (net_exposure / equity) * 100.0
                    } else {
                        0.0
                    })
                    .to_string(),
                    is_basis_points: false,
                }),
                by_asset_class: vec![],
                by_currency: vec![],
                correlated_exposure: vec![],
            }),
            risk: Some(PortfolioRiskMetrics {
                current_drawdown: Some(apex_protos::common::Percentage {
                    value: "0.0".to_string(),
                    is_basis_points: false,
                }),
                max_drawdown_reached: Some(apex_protos::common::Percentage {
                    value: "0.0".to_string(),
                    is_basis_points: false,
                }),
                var_95: None,
                var_99: None,
                expected_shortfall: Some(apex_protos::common::Decimal {
                    value: "0.0".to_string(),
                }),
                beta_to_benchmark: Some(apex_protos::common::Decimal {
                    value: "1.0".to_string(),
                }),
                volatility: Some(apex_protos::common::Decimal {
                    value: "0.0".to_string(),
                }),
                sharpe_ratio: Some(apex_protos::common::Decimal {
                    value: "0.0".to_string(),
                }),
                sortino_ratio: Some(apex_protos::common::Decimal {
                    value: "0.0".to_string(),
                }),
                calmar_ratio: Some(apex_protos::common::Decimal {
                    value: "0.0".to_string(),
                }),
                positions_at_risk_count: 0,
            }),
            positions,
            performance: Some(PerformanceMetrics {
                return_today: Some(apex_protos::common::Percentage {
                    value: (if initial_balance > 0.0 {
                        (realized_pnl / initial_balance) * 100.0
                    } else {
                        0.0
                    })
                    .to_string(),
                    is_basis_points: false,
                }),
                return_this_week: Some(apex_protos::common::Percentage {
                    value: "0.0".to_string(),
                    is_basis_points: false,
                }),
                return_this_month: Some(apex_protos::common::Percentage {
                    value: "0.0".to_string(),
                    is_basis_points: false,
                }),
                win_rate: Some(apex_protos::common::Decimal {
                    value: "0.0".to_string(),
                }),
                profit_factor: Some(apex_protos::common::Decimal {
                    value: "0.0".to_string(),
                }),
                average_winner: Some(apex_protos::common::Decimal {
                    value: "0.0".to_string(),
                }),
                average_loser: Some(apex_protos::common::Decimal {
                    value: "0.0".to_string(),
                }),
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
        info!(
            "get_portfolio_state called for portfolio: {}",
            req.portfolio_id
        );
        let state = self.build_snapshot_from_db(&req.portfolio_id).await?;
        Ok(Response::new(PortfolioStateResponse { state: Some(state) }))
    }

    async fn get_exposure(
        &self,
        request: Request<ExposureQuery>,
    ) -> Result<Response<ExposureResponse>, Status> {
        let req = request.into_inner();
        let snap = self.build_snapshot_from_db(&req.portfolio_id).await?;
        Ok(Response::new(ExposureResponse {
            exposure: snap.exposure,
        }))
    }

    async fn get_heat(
        &self,
        request: Request<HeatQuery>,
    ) -> Result<Response<HeatResponse>, Status> {
        let req = request.into_inner();
        let snap = self.build_snapshot_from_db(&req.portfolio_id).await?;
        let equity = snap
            .equity
            .as_ref()
            .map(|m| m.amount.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);
        let margin_used = snap
            .margin_used
            .as_ref()
            .map(|m| m.amount.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);
        let margin_utilization = if equity > 0.0 {
            (margin_used / equity) * 100.0
        } else {
            0.0
        };
        Ok(Response::new(HeatResponse {
            heat_index: Some(apex_protos::common::Decimal {
                value: "0.0".to_string(),
            }),
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
                    total_drift: Some(apex_protos::common::Decimal {
                        value: "0.0".to_string(),
                    }),
                    positions_needing_rebalance: 0,
                    estimated_turnover: Some(apex_protos::common::Money {
                        amount: "0.0".to_string(),
                        currency: "USD".to_string(),
                        exponent: 0,
                    }),
                }),
            }),
        }))
    }

    async fn get_quality(
        &self,
        request: Request<QualityQuery>,
    ) -> Result<Response<QualityResponse>, Status> {
        let req = request.into_inner();

        let rows = sqlx::query("SELECT realized_pnl FROM positions WHERE state = 'closed'")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut wins = 0;
        let mut losses = 0;
        let mut gross_profit = rust_decimal::Decimal::ZERO;
        let mut gross_loss = rust_decimal::Decimal::ZERO;

        for r in &rows {
            let pnl_opt: Option<rust_decimal::Decimal> = r.get("realized_pnl");
            let pnl = pnl_opt.unwrap_or(rust_decimal::Decimal::ZERO);
            if pnl.is_sign_positive() {
                wins += 1;
                gross_profit += pnl;
            } else if pnl.is_sign_negative() {
                losses += 1;
                gross_loss += pnl.abs();
            }
        }

        let total_trades = wins + losses;
        let win_rate = if total_trades > 0 {
            rust_decimal::Decimal::from(wins) / rust_decimal::Decimal::from(total_trades)
        } else {
            rust_decimal::Decimal::ZERO
        };

        let profit_factor = if gross_loss.is_zero() {
            if gross_profit.is_zero() {
                rust_decimal::Decimal::ONE
            } else {
                rust_decimal::Decimal::from(10)
            }
        } else {
            gross_profit / gross_loss
        };

        let avg_win = if wins > 0 {
            gross_profit / rust_decimal::Decimal::from(wins)
        } else {
            rust_decimal::Decimal::ZERO
        };
        let avg_loss = if losses > 0 {
            gross_loss / rust_decimal::Decimal::from(losses)
        } else {
            rust_decimal::Decimal::ZERO
        };
        let expectancy =
            (win_rate * avg_win) - ((rust_decimal::Decimal::ONE - win_rate) * avg_loss);
        let average_rr = if avg_loss.is_zero() {
            rust_decimal::Decimal::ONE
        } else {
            avg_win / avg_loss
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let quality = crate::quality::quality_score::PortfolioQuality::calculate(
            win_rate,
            profit_factor,
            expectancy,
            average_rr,
            now,
        );

        let _ = self
            .repository
            .store
            .save_quality(
                &req.portfolio_id,
                quality.current_score,
                &serde_json::to_value(&quality.breakdown).unwrap_or_default(),
            )
            .await;

        Ok(Response::new(QualityResponse {
            quality_score: Some(apex_protos::common::Decimal {
                value: quality.current_score.to_string(),
            }),
        }))
    }

    async fn get_health(
        &self,
        request: Request<HealthQuery>,
    ) -> Result<Response<HealthResponse>, Status> {
        let req = request.into_inner();
        let port_state = self
            .registry
            .get_state()
            .map_err(|e| Status::internal(format!("Failed to get portfolio state: {:?}", e)))?;
        let exp_state = self
            .exposure_registry
            .get_state()
            .map_err(|e| Status::internal(format!("Failed to get exposure state: {:?}", e)))?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let health =
            crate::health::health_score::PortfolioHealth::calculate(&port_state, &exp_state, now);

        let _ = self
            .repository
            .store
            .save_health(
                &req.portfolio_id,
                health.current_score as i32,
                &format!("{:?}", health.state),
                &serde_json::to_value(&health.breakdown).unwrap_or_default(),
            )
            .await;

        Ok(Response::new(HealthResponse {
            health_score: Some(apex_protos::common::Decimal {
                value: health.current_score.to_string(),
            }),
            status: format!("{:?}", health.state).to_uppercase(),
        }))
    }

    async fn get_drawdown(
        &self,
        request: Request<DrawdownQuery>,
    ) -> Result<Response<DrawdownResponse>, Status> {
        let req = request.into_inner();
        let snap = self.build_snapshot_from_db(&req.portfolio_id).await?;
        let current_drawdown = snap.risk.as_ref().and_then(|r| r.current_drawdown.clone());
        let max_drawdown = snap
            .risk
            .as_ref()
            .and_then(|r| r.max_drawdown_reached.clone());
        Ok(Response::new(DrawdownResponse {
            current_drawdown,
            max_drawdown,
        }))
    }

    async fn get_correlation(
        &self,
        request: Request<CorrelationQuery>,
    ) -> Result<Response<CorrelationResponse>, Status> {
        let req = request.into_inner();
        let exp_state = self
            .exposure_registry
            .get_state()
            .map_err(|e| Status::internal(format!("Failed to get exposure state: {:?}", e)))?;

        let symbols: Vec<String> = exp_state.symbols.keys().cloned().collect();
        let mut returns_map = std::collections::HashMap::new();
        for sym in &symbols {
            let rows = sqlx::query("SELECT return_percent FROM positions WHERE symbol = $1 AND state = 'closed' ORDER BY updated_at DESC LIMIT 30")
                .bind(sym)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;
            let rets: Vec<rust_decimal::Decimal> = rows
                .iter()
                .map(|r| {
                    r.get::<Option<rust_decimal::Decimal>, _>("return_percent")
                        .unwrap_or_default()
                })
                .collect();
            if rets.len() >= 2 {
                returns_map.insert(sym.clone(), rets);
            }
        }

        let avg_corr_val = if returns_map.len() >= 2 {
            if let Ok(matrix) = crate::correlation::matrix::CorrelationMatrix::from_returns(
                crate::correlation::matrix::CorrelationType::Symbol,
                crate::correlation::matrix::CorrelationWindow::MediumTerm,
                &returns_map,
            ) {
                let mut sum = rust_decimal::Decimal::ZERO;
                let mut count = 0;
                for i in 0..matrix.rows {
                    for j in (i + 1)..matrix.cols {
                        if let Some(c) = matrix.get_correlation(i, j) {
                            sum += c;
                            count += 1;
                        }
                    }
                }
                let avg = if count > 0 {
                    sum / rust_decimal::Decimal::from(count)
                } else {
                    rust_decimal::Decimal::new(15, 2)
                };
                let _ = self
                    .repository
                    .store
                    .save_correlation(
                        &req.portfolio_id,
                        "MediumTerm",
                        "Symbol",
                        &serde_json::to_value(&matrix.identifiers).unwrap_or_default(),
                        &serde_json::to_value(&matrix.data).unwrap_or_default(),
                    )
                    .await;
                avg
            } else {
                rust_decimal::Decimal::new(15, 2)
            }
        } else {
            rust_decimal::Decimal::new(15, 2)
        };

        Ok(Response::new(CorrelationResponse {
            avg_correlation: Some(apex_protos::common::Decimal {
                value: avg_corr_val.to_string(),
            }),
        }))
    }

    async fn get_recommendations(
        &self,
        request: Request<RecommendationsQuery>,
    ) -> Result<Response<RecommendationsResponse>, Status> {
        let req = request.into_inner();
        let exp_state = self
            .exposure_registry
            .get_state()
            .map_err(|e| Status::internal(format!("Failed to get exposure state: {:?}", e)))?;

        let mut current_weights = Vec::new();
        for (sym, exp) in &exp_state.symbols {
            current_weights.push((sym.clone(), exp.weight));
        }

        let row_opt = sqlx::query("SELECT allocations FROM portfolio_allocations WHERE portfolio_id = $1 ORDER BY timestamp DESC LIMIT 1")
            .bind(&req.portfolio_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut targets = Vec::new();
        if let Some(row) = row_opt {
            if let Ok(allocs_val) = row.try_get::<serde_json::Value, _>("allocations") {
                if let Ok(parsed_targets) =
                    serde_json::from_value::<Vec<crate::rebalancing::RebalanceTarget>>(allocs_val)
                {
                    targets = parsed_targets;
                }
            }
        }

        if targets.is_empty() {
            let active_symbols = current_weights.len();
            if active_symbols > 0 {
                let eq_weight =
                    rust_decimal::Decimal::ONE / rust_decimal::Decimal::from(active_symbols);
                for (sym, _) in &current_weights {
                    targets.push(crate::rebalancing::RebalanceTarget {
                        symbol: sym.clone(),
                        target_weight: eq_weight,
                    });
                }
            }
        }

        let engine = crate::rebalancing::RebalanceEngine::new(rust_decimal::Decimal::new(2, 2));
        let actions = engine.calculate_actions(&current_weights, &targets);

        let mut recommendations = Vec::new();
        for act in actions {
            recommendations.push(TradeInstruction {
                symbol: Some(apex_protos::common::Symbol {
                    code: act.symbol.clone(),
                    exchange: "".to_string(),
                    asset_class: 0,
                    description: "".to_string(),
                }),
                side: if act.is_buy {
                    apex_protos::common::TradeSide::Buy as i32
                } else {
                    apex_protos::common::TradeSide::Sell as i32
                },
                volume: Some(apex_protos::common::Volume {
                    units: act.weight_delta.abs().to_string(),
                    lot_size: "100000".to_string(),
                    fractional: true,
                }),
                reason: format!("Adjust target allocation drift by {}", act.weight_delta),
            });
        }

        Ok(Response::new(RecommendationsResponse { recommendations }))
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
        Ok(Response::new(LoadSnapshotResponse { state: Some(snap) }))
    }

    type LoadEventsStream = ReceiverStream<Result<PortfolioEvent, Status>>;

    async fn load_events(
        &self,
        request: Request<LoadEventsRequest>,
    ) -> Result<Response<Self::LoadEventsStream>, Status> {
        let req = request.into_inner();
        let events = if let Some(ts) = req.from {
            let offset_time =
                time::OffsetDateTime::from_unix_timestamp(ts.seconds).map_err(|e| {
                    Status::invalid_argument(format!("Invalid timestamp seconds: {}", e))
                })?;
            let offset_time = offset_time + time::Duration::nanoseconds(ts.nanos as i64);
            self.repository
                .load_events_since_time(&req.portfolio_id, offset_time)
                .await
                .map_err(|e| Status::internal(e.to_string()))?
        } else {
            self.repository
                .load_events_since(&req.portfolio_id, 0)
                .await
                .map_err(|e| Status::internal(e.to_string()))?
        };

        let (tx, rx) = mpsc::channel(100);
        tokio::spawn(async move {
            for e in events {
                if let Ok(proto_event) = serde_json::from_value::<PortfolioEvent>(
                    serde_json::to_value(e.payload).unwrap_or_default(),
                ) {
                    if tx.send(Ok(proto_event)).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn get_equity_curve(
        &self,
        request: Request<EquityCurveQuery>,
    ) -> Result<Response<EquityCurveResponse>, Status> {
        let req = request.into_inner();
        let rows = sqlx::query("SELECT timestamp, payload FROM portfolio_snapshots WHERE aggregate_id = $1 ORDER BY version ASC")
            .bind(&req.portfolio_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut points = Vec::new();
        for r in rows {
            let ts: time::OffsetDateTime = r.get("timestamp");
            let payload: serde_json::Value = r.get("payload");
            let equity = payload
                .get("equity")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string();
            let balance = payload
                .get("balance")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string();
            let drawdown = payload
                .get("drawdown")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string();
            points.push(PortfolioDataPoint {
                timestamp: Some(apex_protos::common::Timestamp {
                    seconds: ts.unix_timestamp(),
                    nanos: 0,
                }),
                equity: Some(apex_protos::common::Money {
                    amount: equity,
                    currency: "USD".to_string(),
                    exponent: 0,
                }),
                balance: Some(apex_protos::common::Money {
                    amount: balance,
                    currency: "USD".to_string(),
                    exponent: 0,
                }),
                margin_used: None,
                unrealized_pnl: None,
                position_count: 0,
                drawdown_percent: Some(apex_protos::common::Decimal { value: drawdown }),
            });
        }
        Ok(Response::new(EquityCurveResponse { points }))
    }

    async fn get_symbol_performance(
        &self,
        request: Request<SymbolPerformanceQuery>,
    ) -> Result<Response<SymbolPerformanceResponse>, Status> {
        let req = request.into_inner();
        let row_opt = sqlx::query("SELECT COALESCE(SUM(realized_pnl), 0) as realized, COUNT(*) as count FROM positions WHERE symbol = $1 AND state = 'closed'")
            .bind(&req.symbol)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let (realized, _count) = if let Some(row) = row_opt {
            let pnl: rust_decimal::Decimal = row.get("realized");
            let cnt: i64 = row.get("count");
            (pnl, cnt)
        } else {
            (rust_decimal::Decimal::ZERO, 0)
        };

        let return_pct_val = if realized.is_zero() {
            "0.0".to_string()
        } else {
            (realized / rust_decimal::Decimal::from(100000)).to_string()
        };

        Ok(Response::new(SymbolPerformanceResponse {
            return_pct: Some(apex_protos::common::Percentage {
                value: return_pct_val,
                is_basis_points: false,
            }),
            pnl: Some(apex_protos::common::Money {
                amount: realized.to_string(),
                currency: "USD".to_string(),
                exponent: 0,
            }),
        }))
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
        let (tx, rx) = mpsc::channel(100);
        if let Ok(state) = self.build_snapshot_from_db(&_req.client_id).await {
            let _ = tx.send(Ok(state)).await;
        }
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
