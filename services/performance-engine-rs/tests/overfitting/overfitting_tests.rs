#[cfg(test)]
mod overfitting_tests {
    use rust_decimal_macros::dec;
    use performance_engine::overfitting::overfit_detector::{OverfitDetector, OverfitInput, OverfitState};
    use performance_engine::overfitting::sample_bias::{SampleBiasDetector, sample_size_penalty};
    use performance_engine::overfitting::confidence_penalty::ConfidencePenaltyEngine;

    // ─── SampleBias ────────────────────────────────────────────────────────────

    #[test]
    fn sample_bias_insufficient() {
        let r = SampleBiasDetector::evaluate(10);
        assert!(r.is_biased);
        assert_eq!(r.penalty_multiplier, dec!(0.10));
    }

    #[test]
    fn sample_bias_institutional() {
        let r = SampleBiasDetector::evaluate(500);
        assert!(!r.is_biased);
        assert_eq!(r.penalty_multiplier, dec!(1.0));
    }

    #[test]
    fn sample_penalty_monotone() {
        let p10  = sample_size_penalty(10);
        let p30  = sample_size_penalty(30);
        let p75  = sample_size_penalty(75);
        let p150 = sample_size_penalty(150);
        let p400 = sample_size_penalty(400);
        assert!(p10 < p30);
        assert!(p30 < p75);
        assert!(p75 <= p150);
        assert!(p150 <= p400);
    }

    // ─── ConfidencePenalty ─────────────────────────────────────────────────────

    #[test]
    fn confidence_penalty_all_perfect() {
        let r = ConfidencePenaltyEngine::compute(dec!(1), dec!(0), dec!(1));
        assert_eq!(r.combined_penalty, dec!(1));
    }

    #[test]
    fn confidence_penalty_clamped_above_zero() {
        let r = ConfidencePenaltyEngine::compute(dec!(0), dec!(1), dec!(0));
        assert!(r.combined_penalty >= dec!(0));
    }

    #[test]
    fn confidence_penalty_applied_correctly() {
        let r = ConfidencePenaltyEngine::compute(dec!(0.8), dec!(0.2), dec!(0.9));
        let adjusted = ConfidencePenaltyEngine::apply(dec!(0.9), &r);
        assert!(adjusted <= dec!(0.9), "Adjusted confidence must be <= raw");
        assert!(adjusted >= dec!(0));
    }

    // ─── OverfitDetector ───────────────────────────────────────────────────────

    #[test]
    fn overfit_healthy_on_strong_oos() {
        let input = OverfitInput {
            parameters_tested: 3,
            in_sample_trades: 200,
            out_of_sample_trades: 100,
            in_sample_expectancy: dec!(0.12),
            out_of_sample_expectancy: dec!(0.11),
            in_sample_pf: dec!(2.0),
            out_of_sample_pf: dec!(1.9),
        };
        let r = OverfitDetector::evaluate(&input);
        assert!(
            r.state == OverfitState::Healthy || r.state == OverfitState::Caution,
            "Got {:?}", r.state
        );
    }

    #[test]
    fn overfit_dangerous_on_bad_oos() {
        let input = OverfitInput {
            parameters_tested: 500,
            in_sample_trades: 50,
            out_of_sample_trades: 10,    // tiny OOS
            in_sample_expectancy: dec!(0.20),
            out_of_sample_expectancy: dec!(0.05), // collapsed OOS
            in_sample_pf: dec!(3.0),
            out_of_sample_pf: dec!(1.1),
        };
        let r = OverfitDetector::evaluate(&input);
        assert!(
            r.state == OverfitState::Overfit || r.state == OverfitState::Dangerous,
            "Got {:?}", r.state
        );
        assert!(r.confidence_penalty < dec!(0.7));
    }

    #[test]
    fn overfit_penalty_never_exceeds_one() {
        for i in 0..100u32 {
            let input = OverfitInput {
                parameters_tested: i,
                in_sample_trades: 100,
                out_of_sample_trades: 50,
                in_sample_expectancy: dec!(0.10),
                out_of_sample_expectancy: dec!(0.10),
                in_sample_pf: dec!(2.0),
                out_of_sample_pf: dec!(2.0),
            };
            let r = OverfitDetector::evaluate(&input);
            assert!(r.confidence_penalty <= dec!(1));
            assert!(r.confidence_penalty >= dec!(0));
        }
    }
}
