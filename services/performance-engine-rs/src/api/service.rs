use apex_protos::analytics::analytics_engine_server::AnalyticsEngine as GrpcAnalyticsEngine;
use apex_protos::analytics::{
    DrawdownAnalysis, DrawdownQuery, EquityCurve, EquityCurveQuery, MetricsQuery,
    MonteCarloRequest, MonteCarloResults, MonthlyReturns, MonthlyReturnsQuery, PerformanceQuery,
    PerformanceReport, ReturnMetrics, RiskMetrics, StrategyMetrics, TradeMetrics, TradeStatistics,
    TradeStatsQuery,
};
use apex_protos::common::{Empty, Money, Result as CommonResult};
use rust_decimal::Decimal;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use crate::analytics::engine::AnalyticsEngine;
use crate::storage::Repositories;
use crate::validation::monte_carlo::PerformanceMonteCarlo;

#[derive(Clone, Default)]
pub struct PerformanceState {
    pub sharpe_ratio: Decimal,
    pub sortino_ratio: Decimal,
    pub net_profit: Decimal,
    pub win_rate: Decimal,
}

pub struct AnalyticsServiceImpl {
    pub state: Arc<RwLock<PerformanceState>>,
    pub repos: Arc<Repositories>,
    pub engine: Arc<AnalyticsEngine>,
}

impl AnalyticsServiceImpl {
    pub fn new(repos: Arc<Repositories>, engine: Arc<AnalyticsEngine>) -> Self {
        Self {
            state: Arc::new(RwLock::new(PerformanceState::default())),
            repos,
            engine,
        }
    }

    fn to_proto_decimal(d: rust_decimal::Decimal) -> apex_protos::common::Decimal {
        apex_protos::common::Decimal {
            value: d.to_string(),
        }
    }

    fn to_proto_money(d: rust_decimal::Decimal) -> Money {
        Money {
            amount: d.to_string(),
            currency: "USD".to_string(),
            exponent: 2,
        }
    }
}

#[tonic::async_trait]
impl GrpcAnalyticsEngine for AnalyticsServiceImpl {
    async fn get_performance_report(
        &self,
        request: Request<PerformanceQuery>,
    ) -> std::result::Result<Response<PerformanceReport>, Status> {
        let query = request.into_inner();

        // Fetch trades
        let from = query.from.map(|t| {
            chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32).unwrap_or_default()
        });
        let to = query.to.map(|t| {
            chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32).unwrap_or_default()
        });

        let trades = self
            .repos
            .performance
            .get_trades_for_strategy(&query.strategy_id, from, to)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if trades.is_empty() {
            return Ok(Response::new(PerformanceReport {
                report_id: uuid::Uuid::new_v4().to_string(),
                strategy_id: query.strategy_id,
                period: None,
                returns: None,
                risk: None,
                trades: None,
                time: None,
                benchmark: None,
                distribution: None,
                periods: vec![],
            }));
        }

        // Compute via engine
        let result =
            crate::analytics::engine::AnalyticsEngine::compute(&query.strategy_id, &trades);

        let returns = ReturnMetrics {
            total_return: Some(Self::to_proto_decimal(result.net_profit)),
            annualized_return: Some(Self::to_proto_decimal(result.expectancy)),
            cagr: Some(Self::to_proto_decimal(result.expectancy)),
            daily_average_return: None,
            weekly_average_return: None,
            monthly_average_return: None,
            return_volatility: Some(Self::to_proto_decimal(Decimal::ZERO)), // TODO: track if needed
            annualized_volatility: Some(Self::to_proto_decimal(Decimal::ZERO)),
        };

        let risk = RiskMetrics {
            max_drawdown: Some(Self::to_proto_decimal(result.max_drawdown)),
            max_drawdown_duration_days: None,
            average_drawdown: Some(Self::to_proto_decimal(result.average_drawdown)),
            recovery_factor: Some(Self::to_proto_decimal(result.recovery_factor)),
            volatility: Some(Self::to_proto_decimal(Decimal::ZERO)),
            downside_deviation: Some(Self::to_proto_decimal(Decimal::ZERO)),
            var: None,
            expected_shortfall: None,
            calmar_ratio: Some(Self::to_proto_decimal(result.calmar_ratio)),
            sterling_ratio: Some(Self::to_proto_decimal(result.sterling_ratio)),
            ulcer_index: Some(Self::to_proto_decimal(result.ulcer_index)),
            ulcer_performance_index: 0,
        };

        let trade_metrics = TradeMetrics {
            total_trades: result.trade_count as u32,
            winning_trades: result.win_count as u32,
            losing_trades: result.loss_count as u32,
            break_even_trades: result.breakeven_count as u32,
            win_rate: Some(Self::to_proto_decimal(result.win_rate)),
            loss_rate: Some(Self::to_proto_decimal(result.loss_rate)),
            profit_factor: Some(Self::to_proto_decimal(result.profit_factor)),
            payback_ratio: None,
            expectancy: Some(Self::to_proto_decimal(result.expectancy)),
            sqn: Some(Self::to_proto_decimal(result.sqn)),
            average_trade: None,
            average_winner: Some(Self::to_proto_money(result.average_win)),
            average_loser: Some(Self::to_proto_money(result.average_loss)),
            average_win_loss_ratio: None,
            largest_winner: Some(Self::to_proto_decimal(result.largest_win)),
            largest_loser: Some(Self::to_proto_decimal(result.largest_loss)),
            streaks: None,
            consecutive: None,
        };

        Ok(Response::new(PerformanceReport {
            report_id: uuid::Uuid::new_v4().to_string(),
            strategy_id: query.strategy_id,
            period: None,
            returns: Some(returns),
            risk: Some(risk),
            trades: Some(trade_metrics),
            time: None,
            benchmark: None,
            distribution: None,
            periods: vec![],
        }))
    }

    async fn get_trade_statistics(
        &self,
        request: Request<TradeStatsQuery>,
    ) -> std::result::Result<Response<TradeStatistics>, Status> {
        let query = request.into_inner();
        let aggregates = self
            .repos
            .performance
            .aggregate_by_dimension(&query.strategy_id, &query.group_by)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut groups = Vec::new();
        let mut total_trades = 0;

        for agg in aggregates {
            total_trades += agg.trade_count;
            groups.push(apex_protos::analytics::TradeGroup {
                group_key: agg.group_key,
                trade_count: agg.trade_count as u32,
                wins: agg.wins as u32,
                losses: agg.losses as u32,
                win_rate: Some(Self::to_proto_decimal(if agg.trade_count > 0 {
                    Decimal::from(agg.wins) / Decimal::from(agg.trade_count)
                } else {
                    Decimal::ZERO
                })),
                gross_profit: None,
                gross_loss: None,
                net_pnl: None,
                profit_factor: None,
                average_pnl: None,
                return_percent: None,
                max_drawdown: None,
            });
        }

        Ok(Response::new(TradeStatistics {
            strategy_id: query.strategy_id,
            total_trades: total_trades as u32,
            groups,
            best_symbol: None,
            worst_symbol: None,
        }))
    }

    async fn get_drawdown_analysis(
        &self,
        _request: Request<DrawdownQuery>,
    ) -> std::result::Result<Response<DrawdownAnalysis>, Status> {
        // Implement full drawdown analysis later, for now return empty
        Ok(Response::new(DrawdownAnalysis::default()))
    }

    async fn get_equity_curve(
        &self,
        _request: Request<EquityCurveQuery>,
    ) -> std::result::Result<Response<EquityCurve>, Status> {
        Ok(Response::new(EquityCurve::default()))
    }

    async fn calculate_metrics(
        &self,
        request: Request<MetricsQuery>,
    ) -> std::result::Result<Response<StrategyMetrics>, Status> {
        let query = request.into_inner();
        let snapshot = self
            .repos
            .statistics
            .get_latest_snapshot(&query.strategy_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if let Some(snap) = snapshot {
            Ok(Response::new(StrategyMetrics {
                strategy_id: query.strategy_id,
                calculated_at: Some(apex_protos::common::Timestamp {
                    seconds: snap.computed_at.timestamp(),
                    nanos: snap.computed_at.timestamp_subsec_nanos() as i32,
                }),
                sharpe_ratio: Some(Self::to_proto_decimal(snap.sharpe_ratio)),
                sortino_ratio: Some(Self::to_proto_decimal(snap.sortino_ratio)),
                omega_ratio: Some(Self::to_proto_decimal(snap.omega_ratio)),
                kappa_ratio: None,
                calmar_ratio: Some(Self::to_proto_decimal(snap.calmar_ratio)),
                decile_calmar_ratio: None,
                burke_ratio: None,
                tail_ratio: None,
                sterling_ratio: None,
                pain_ratio: None,
                up_capture_ratio: None,
                down_capture_ratio: None,
                up_number_ratio: None,
                down_number_ratio: None,
            }))
        } else {
            Ok(Response::new(StrategyMetrics::default()))
        }
    }

    async fn get_monthly_returns(
        &self,
        request: Request<MonthlyReturnsQuery>,
    ) -> std::result::Result<Response<MonthlyReturns>, Status> {
        let query = request.into_inner();
        let aggs = self
            .repos
            .performance
            .get_monthly_returns(&query.strategy_id, query.year)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut months = Vec::new();
        for agg in aggs {
            months.push(apex_protos::analytics::MonthlyReturn {
                month: agg.month,
                return_percent: None, // Missing from DB agg currently
                cumulative_return: None,
                trade_count: agg.trade_count as u32,
                positive: agg.net_pnl > Decimal::ZERO,
            });
        }

        Ok(Response::new(MonthlyReturns {
            strategy_id: query.strategy_id,
            year: query.year,
            months,
            ytd: None,
            stats: None,
        }))
    }

    async fn run_monte_carlo(
        &self,
        request: Request<MonteCarloRequest>,
    ) -> std::result::Result<Response<MonteCarloResults>, Status> {
        let req = request.into_inner();
        let trades = self
            .repos
            .performance
            .get_trades_for_strategy(&req.strategy_id, None, None)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let mc = PerformanceMonteCarlo::new(12345); // Seeded deterministic
        let result = mc.simulate(
            req.simulations as u64,
            &trades,
            rust_decimal_macros::dec!(0.5),
        );

        Ok(Response::new(MonteCarloResults {
            strategy_id: req.strategy_id,
            simulations_run: result.total_trials as i32,
            terminal_equity: Some(apex_protos::analytics::MonteCarloDistribution {
                mean: None,
                median: None,
                std_dev: None,
                skewness: None,
                kurtosis: None,
                percentiles: Some(apex_protos::analytics::Percentiles {
                    p1: None,
                    p5: Some(Self::to_proto_decimal(result.terminal_equity_05)),
                    p10: None,
                    p25: None,
                    p50: None,
                    p75: None,
                    p90: None,
                    p95: Some(Self::to_proto_decimal(result.terminal_equity_95)),
                    p99: None,
                }),
            }),
            max_drawdown: Some(apex_protos::analytics::MonteCarloDistribution {
                mean: None,
                median: Some(Self::to_proto_decimal(result.median_drawdown)),
                std_dev: None,
                skewness: None,
                kurtosis: None,
                percentiles: Some(apex_protos::analytics::Percentiles {
                    p1: None,
                    p5: None,
                    p10: None,
                    p25: None,
                    p50: None,
                    p75: None,
                    p90: None,
                    p95: Some(Self::to_proto_decimal(result.max_drawdown_95)),
                    p99: None,
                }),
            }),
            profit: None,
            risk_of_ruin: Some(apex_protos::analytics::RiskOfRuin {
                probability: Some(Self::to_proto_decimal(result.collapse_probability)),
                ruin_threshold: Some(Self::to_proto_decimal(rust_decimal_macros::dec!(0.5))),
                median_time_to_ruin: 0,
            }),
            confidence: None,
            scenarios: vec![],
        }))
    }

    async fn health(
        &self,
        _request: Request<Empty>,
    ) -> std::result::Result<Response<CommonResult>, Status> {
        let db_ok = self.repos.performance.health_check().await;

        Ok(Response::new(CommonResult {
            ok: db_ok,
            error: if db_ok {
                None
            } else {
                Some(apex_protos::common::Error {
                    code: "DB_UNHEALTHY".to_string(),
                    message: "Database unhealthy".to_string(),
                    severity: 0,
                    details: std::collections::HashMap::new(),
                    causes: vec![],
                })
            },
        }))
    }
}
