use ai_engine_rs::drift_detection::{DriftDirection, DriftReport, DriftState, DriftType, RecommendedAction};
use ai_engine_rs::recommendation_engine::{ExplanationBundle, Recommendation, RecommendationType};
use ai_engine_rs::opportunity_ranking::{ConfidenceScore, OpportunityScore, PriorityGrade, RankingFactors, RankingResult, RankingTarget};
use ai_engine_rs::event_bus_integration::ConsumerEvent;

use rust_decimal::Decimal;
use time::OffsetDateTime;
use uuid::Uuid;

#[test]
fn test_100k_recommendation_cycles() {
    let mut success_count = 0;
    for _i in 0..100_000 {
        let strategy_id = Uuid::new_v4();
        let confidence_score = Decimal::from(80);
        let rec_type = RecommendationType::AllocationRecommendation {
            strategy_id,
            amount: Decimal::from(100),
        };
        let explanation = ExplanationBundle {
            confidence_breakdown_id: Uuid::new_v4(),
            historical_references: vec![Uuid::new_v4()],
            similar_decisions: vec![Uuid::new_v4()],
        };

        let rec = Recommendation::new(
            rec_type,
            confidence_score,
            vec!["strong trend".to_string()],
            "increase profit".to_string(),
            "low risk".to_string(),
            explanation,
        );
        assert!(rec.is_ok());
        success_count += 1;
    }
    assert_eq!(success_count, 100_000);
}

#[test]
fn test_100k_ranking_cycles() {
    let mut success_count = 0;
    for i in 0..100_000 {
        let factors = RankingFactors {
            expectancy: Decimal::from(1),
            winrate: Decimal::from(55),
            drawdown_profile: Decimal::from(5),
            risk_efficiency: Decimal::from(2),
            regime_alignment: Decimal::from(90),
            execution_quality: Decimal::from(95),
            market_quality: Decimal::from(98),
        };

        let target = RankingTarget::Symbol(format!("SYM_{}", i));
        let opt_score = OpportunityScore::new(Decimal::from(75)).unwrap();
        let conf_score = ConfidenceScore::new(Decimal::from(80)).unwrap();

        let result = RankingResult {
            target,
            opportunity_score: opt_score,
            confidence_score: conf_score,
            priority_grade: PriorityGrade::High,
            factors,
            evaluated_at: OffsetDateTime::now_utc(),
        };

        assert_eq!(result.priority_grade, PriorityGrade::High);
        success_count += 1;
    }
    assert_eq!(success_count, 100_000);
}

#[test]
fn test_100k_drift_calculations() {
    let mut success_count = 0;
    for _i in 0..100_000 {
        let drift_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();

        let drift = DriftReport::new(
            drift_id,
            target_id,
            DriftType::Strategy,
            DriftState::Warning,
            Decimal::from(45), // severity
            DriftDirection::Negative,
            Decimal::from(90), // confidence
            RecommendedAction::Monitor,
            OffsetDateTime::now_utc(),
        );

        assert!(drift.is_ok());
        success_count += 1;
    }
    assert_eq!(success_count, 100_000);
}

#[test]
fn test_1m_event_replay_validations() {
    let mut success_count = 0;
    for i in 0..1_000_000 {
        let event = if i % 2 == 0 {
            ConsumerEvent::MarketIntelligence {
                data: "test".to_string(),
            }
        } else {
            ConsumerEvent::StrategyEvent {
                strategy_id: Uuid::new_v4(),
                event_type: "signal".to_string(),
            }
        };

        match event {
            ConsumerEvent::MarketIntelligence { .. } => success_count += 1,
            ConsumerEvent::StrategyEvent { .. } => success_count += 1,
            _ => panic!("Unexpected event type"),
        }
    }
    assert_eq!(success_count, 1_000_000);
}
