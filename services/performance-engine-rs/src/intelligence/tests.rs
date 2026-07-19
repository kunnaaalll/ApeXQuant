use crate::intelligence::*;
#[cfg(test)]
use rust_decimal_macros::dec;

#[test]
fn test_pattern_intelligence_small_sample_penalty() {
    let assessment = PatternAssessment::evaluate(PatternEvaluateParams {
        pattern_name: "TestPattern".to_string(),
        trade_count: 15,
        wins: 10,
        _losses: 5,
        gross_profit: dec!(100),
        gross_loss: dec!(50),
        average_win: dec!(10),
        average_loss: dec!(10),
        max_drawdown: dec!(10),
    });
    assert_eq!(assessment.sample_quality, "Insufficient");
    assert!(assessment.confidence <= dec!(0.1));
}

#[test]
fn test_determinism_loop_pattern() {
    let mut prev_assessment: Option<PatternAssessment> = None;
    for _ in 0..100_000 {
        let assessment = PatternAssessment::evaluate(PatternEvaluateParams {
            pattern_name: "TestPattern".to_string(),
            trade_count: 150,
            wins: 100,
            _losses: 50,
            gross_profit: dec!(1000),
            gross_loss: dec!(500),
            average_win: dec!(10),
            average_loss: dec!(10),
            max_drawdown: dec!(10),
        });
        if let Some(prev) = &prev_assessment {
            assert_eq!(assessment.expectancy, prev.expectancy);
            assert_eq!(assessment.confidence, prev.confidence);
            assert_eq!(assessment.state, prev.state);
        }
        prev_assessment = Some(assessment);
    }
}

#[test]
fn test_edge_intelligence_critical() {
    let edge = EdgeIntelligence::evaluate(dec!(0.1), dec!(0.5));
    assert_eq!(edge.edge_state, EdgeState::Critical);
    assert!(edge.degrading);
}
