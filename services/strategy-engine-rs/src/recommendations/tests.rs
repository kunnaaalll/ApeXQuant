use super::*;
use rust_decimal_macros::dec;

#[test]
fn test_reason_codes() {
    let mut engine = RecommendationEngine::new();

    let rec = engine.generate(dec!(0.8), dec!(0.1), dec!(0.9));
    assert_eq!(rec.action, RecommendationAction::Increase);
    assert!(rec.reason_codes.contains(&ReasonCode::EdgeEmerging));
    assert!(rec.reason_codes.contains(&ReasonCode::ExcellentStability));
}

#[test]
fn test_recommendation_priority() {
    let mut engine = RecommendationEngine::new();

    // High risk should override everything
    let rec = engine.generate(dec!(1.0), dec!(0.9), dec!(1.0));
    assert_eq!(rec.action, RecommendationAction::Retire);
    assert!(rec.reason_codes.contains(&ReasonCode::ExcessiveRisk));

    // Mid risk / collapsing edge -> Pause
    let rec = engine.generate(dec!(-0.3), dec!(0.4), dec!(0.5));
    assert_eq!(rec.action, RecommendationAction::Pause);
    assert!(rec.reason_codes.contains(&ReasonCode::EdgeCollapsing));
}
