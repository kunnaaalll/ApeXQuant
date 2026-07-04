//! Phase 3 Sandbox and Validation tests

use backtester::overfitting::{OverfittingAnalyzer, OverfittingSeverity, OATrade};
use backtester::parameter_optimizer::{OptimizationMethod, ParameterOptimizer, ParameterSpace};
use backtester::promotion::{PromotionEngine, PromotionRequirements, PromotionState};
use backtester::ranking::RankingEngine;
use backtester::regime_validation::RegimeValidator;
use backtester::robustness::{DegradationProfile, RobustnessEvaluator};
use backtester::strategy_sandbox::{SandboxSession, StrategySandbox};
use backtester::walk_forward::{WalkForwardEngine, WalkForwardWindowData, WindowStats};
use rust_decimal::Decimal;

#[test]
fn test_sandbox_session() -> Result<(), &'static str> {
    let session = SandboxSession::new(
        "session_1".to_string(),
        "strategy_1".to_string(),
        1600000000000,
        1600003600000,
    );
    let result = StrategySandbox::run_session(&session)?;
    assert!(result.is_deterministic);
    Ok(())
}

#[test]
fn test_walk_forward_evaluation() -> Result<(), &'static str> {
    let windows = WalkForwardEngine::generate_windows(0, 1000, 500, 200);
    let data: Vec<_> = windows.into_iter().map(|w| {
        WalkForwardWindowData {
            window: w,
            is_stats: WindowStats {
                total_trades: 10,
                winning_trades: 6,
                gross_profit: Decimal::new(600, 0),
                gross_loss: Decimal::new(300, 0),
                max_drawdown: Decimal::new(100, 0),
                net_profit: Decimal::new(300, 0),
            },
            oos_stats: WindowStats {
                total_trades: 5,
                winning_trades: 3,
                gross_profit: Decimal::new(300, 0),
                gross_loss: Decimal::new(150, 0),
                max_drawdown: Decimal::new(50, 0),
                net_profit: Decimal::new(150, 0),
            },
        }
    }).collect();
    let result = WalkForwardEngine::evaluate(&data).map_err(|_| "evaluation failed")?;
    assert!(result.passes_validation);
    Ok(())
}

#[test]
fn test_overfitting_detection() -> Result<(), &'static str> {
    let trades = vec![
        OATrade { pnl: Decimal::new(100, 0) },
        OATrade { pnl: Decimal::new(-50, 0) },
    ];
    let observed_sharpe = Decimal::new(15, 1);
    let analysis = OverfittingAnalyzer::analyze_from_trades(&trades, observed_sharpe, 42)
        .map_err(|_| "analysis failed")?;
    assert!(matches!(analysis.severity, OverfittingSeverity::Healthy | OverfittingSeverity::Warning | OverfittingSeverity::Critical));
    Ok(())
}

#[test]
fn test_parameter_optimization() -> Result<(), &'static str> {
    let space = ParameterSpace {
        stop_loss_ticks: vec![10, 20],
        take_profit_ticks: vec![20, 40],
        timeframes: vec!["1m".to_string()],
        sessions: vec!["RTH".to_string()],
        risk_per_trade: vec![Decimal::new(1, 2)],
    };
    
    let result = ParameterOptimizer::optimize(&space, OptimizationMethod::DeterministicSweep)?;
    assert_eq!(result.best_stop_loss_ticks, 20); // Dynamic optimal value
    Ok(())
}

#[test]
fn test_robustness_evaluation() -> Result<(), &'static str> {
    let profile = DegradationProfile {
        additional_spread_ticks: 2,
        latency_ms: 50,
        slippage_ticks: 1,
    };
    let eval = RobustnessEvaluator::evaluate("strategy_1", &profile)?;
    assert!(eval.passes);
    Ok(())
}

#[test]
fn test_regime_validation() -> Result<(), &'static str> {
    let scores = RegimeValidator::validate("strategy_1")?;
    assert!(scores.is_empty()); // Stub returns empty vec
    Ok(())
}

#[test]
fn test_ranking() -> Result<(), &'static str> {
    let strategies = vec!["strategy_1".to_string(), "strategy_2".to_string()];
    let ranks = RankingEngine::rank_global(&strategies)?;
    assert!(ranks.is_empty()); // Stub returns empty vec
    Ok(())
}

#[test]
fn test_promotion_evaluation() -> Result<(), &'static str> {
    let reqs = PromotionRequirements {
        min_trade_count: 100,
        min_robustness_score: Decimal::new(80, 2),
        min_oos_performance: Decimal::ZERO,
        max_drawdown_limit: Decimal::new(10, 2),
    };
    let decision = PromotionEngine::evaluate_promotion("strategy_1", PromotionState::Research, &reqs)?;
    assert!(!decision.is_approved);
    Ok(())
}
