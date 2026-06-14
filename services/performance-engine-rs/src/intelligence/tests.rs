#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use crate::intelligence::*;

    #[test]
    fn test_pattern_intelligence_small_sample_penalty() {
        let assessment = PatternAssessment::evaluate(
            "TestPattern".to_string(),
            15,
            10,
            5,
            dec!(100),
            dec!(50),
            dec!(10),
            dec!(10),
            dec!(0.1)
        );
        assert_eq!(assessment.sample_quality, "Insufficient");
        assert!(assessment.confidence <= dec!(0.1));
    }

    #[test]
    fn test_determinism_loop_pattern() {
        let mut prev_assessment: Option<PatternAssessment> = None;
        for _ in 0..100_000 {
            let assessment = PatternAssessment::evaluate(
                "TestPattern".to_string(),
                150,
                100,
                50,
                dec!(1000),
                dec!(500),
                dec!(10),
                dec!(10),
                dec!(0.1)
            );
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
}
