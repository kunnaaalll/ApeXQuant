use rust_decimal_macros::dec;
use backtester::parity::{ParityState, ParityReport, MarketParityValidator, ParityValidator};
use backtester::shadow_validation::{ShadowExecution, ShadowValidator};
use backtester::certification::{CertificationEngine, CertificationStage, PromotionCriteria, StrategyMetrics};

#[test]
fn test_parity_validations_stress() {
    let validator = MarketParityValidator::new(dec!(0.05));
    
    // Simulating 100,000 parity comparisons
    let mut valid_count = 0;
    for _ in 0..100_000 {
        // Dummy data for stress test
        let live_price = dec!(100.0);
        let sim_price = dec!(100.01);
        
        let report = validator.validate(&live_price, &sim_price);
        if report.is_valid {
            valid_count += 1;
        }
    }
    
    assert_eq!(valid_count, 100_000);
}



#[test]
fn test_trade_validations_stress() {
    let validator = ShadowValidator::new();
    
    let live = ShadowExecution {
        order_id: "ORD_1".to_string(),
        fill_price: dec!(150.0),
        fill_size: dec!(10.0),
        latency_ms: 15,
        slippage: dec!(0.01),
        risk_interventions: 0,
    };
    
    let shadow = ShadowExecution {
        order_id: "ORD_1".to_string(),
        fill_price: dec!(150.0),
        fill_size: dec!(10.0),
        latency_ms: 20,
        slippage: dec!(0.02),
        risk_interventions: 0,
    };
    
    let backtest = ShadowExecution {
        order_id: "ORD_1".to_string(),
        fill_price: dec!(150.0),
        fill_size: dec!(10.0),
        latency_ms: 10,
        slippage: dec!(0.0),
        risk_interventions: 0,
    };

    // 1,000,000 validations
    let mut total_latency_diff = 0;
    for _ in 0..1_000_000 {
        let report = validator.validate(&shadow, &live, &backtest);
        total_latency_diff += report.latency_diff_ms;
    }
    
    assert_eq!(total_latency_diff, 5_000_000); // (20 - 15) * 1,000,000 = 5_000_000
}

#[test]
fn test_certification_cycles_stress() {
    let criteria = PromotionCriteria {
        min_parity_threshold: dec!(90.0),
        min_robustness_threshold: dec!(80.0),
        min_trades: 100,
        min_months: 3,
        max_drift: dec!(5.0),
    };
    let engine = CertificationEngine::new(criteria);

    let metrics = StrategyMetrics {
        current_parity: dec!(95.0),
        current_robustness: dec!(85.0),
        total_trades: 150,
        active_months: 4,
        current_drift: dec!(2.0),
    };

    let mut promoted_count = 0;
    for _ in 0..10_000 {
        let (eligible, next_stage) = engine.evaluate_promotion(CertificationStage::Candidate, &metrics);
        if eligible && next_stage == CertificationStage::Production {
            promoted_count += 1;
        }
    }
    
    assert_eq!(promoted_count, 10_000);
}
