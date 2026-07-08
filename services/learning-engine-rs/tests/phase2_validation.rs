use learning_engine::adaptation::{AdaptationResult, AdaptationState};
use learning_engine::promotion::{PromotionLadder, PromotionLevel, StrategyPerformance};
use learning_engine::retirement::{RetirementAction, RetirementManager};
use learning_engine::anomaly::{AnomalyDetector, AnomalySeverity};
use learning_engine::drift::{DriftMonitor, DriftMetrics, DriftStatus};
use learning_engine::regime_memory::{RegimeMemory, MarketRegime};
use learning_engine::clustering::ClusterManager;
use rust_decimal::Decimal;
use std::collections::HashMap;

#[test]
fn test_promotion_progression() {
    let ladder = PromotionLadder::new();
    let perf = StrategyPerformance {
        sample_size: 501,
        confidence: Decimal::new(75, 0),
        current_drawdown: Decimal::new(5, 0),
        regime_robustness: Decimal::new(65, 0),
    };

    let next = ladder.evaluate_promotion(PromotionLevel::Shadow, &perf).unwrap();
    assert_eq!(next, Some(PromotionLevel::Candidate));
}

#[test]
fn test_retirement_triggers() {
    let manager = RetirementManager::new();
    let action = manager.evaluate(
        Decimal::new(5, 0), // below 10 edge
        Decimal::new(20, 0), // below 30 confidence
        60, // over 50 duration
        false, // regime drift
    );
    // 3 triggers -> Retire
    assert_eq!(action, Some(RetirementAction::Retire));
}

#[test]
fn test_regime_replay() {
    let mut memory = RegimeMemory::new();
    memory.record_success(MarketRegime::Trend, "strat_1".to_string());
    
    let record = memory.get_record(&MarketRegime::Trend).unwrap();
    assert!(record.successful_strategies.contains(&"strat_1".to_string()));
}

#[test]
fn test_anomaly_detection() {
    let detector = AnomalyDetector::new();
    let anomaly = detector.detect_slippage_anomaly(Decimal::new(10, 0), Decimal::new(1, 0), Decimal::new(1, 0)).unwrap();
    assert_eq!(anomaly.severity, AnomalySeverity::Critical);
}

#[test]
fn test_drift_detection() {
    let monitor = DriftMonitor::new();
    let strategy = DriftMetrics {
        current_deviation: Decimal::new(15, 0),
        warning_threshold: Decimal::new(10, 0),
        critical_threshold: Decimal::new(20, 0),
    };
    let market = strategy.clone();
    let execution = strategy.clone();
    let risk = strategy.clone();

    let analysis = monitor.analyze(strategy, market, execution, risk);
    assert_eq!(analysis.strategy_drift, DriftStatus::Warning);
}

#[test]
fn test_clustering_deterministic() {
    let manager = ClusterManager::new();
    let mut traits = HashMap::new();
    traits.insert("momentum".to_string(), vec!["s1".to_string(), "s2".to_string()]);
    let clusters = manager.cluster_strategies(&traits);
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].id, "cluster_momentum");
}

#[test]
fn stress_test_100k_adaptation_cycles() {
    // 100,000 adaptation cycles
    for i in 0..100_000 {
        let mut adapt = AdaptationResult::new(format!("strat_{}", i));
        adapt.metrics.regime_adaptation = Decimal::new(80, 0);
        adapt.metrics.volatility_adaptation = Decimal::new(70, 0);
        adapt.metrics.session_adaptation = Decimal::new(90, 0);
        adapt.metrics.symbol_adaptation = Decimal::new(60, 0);
        
        adapt.calculate_score().unwrap();
        assert_eq!(adapt.state, AdaptationState::Stable);
    }
}
