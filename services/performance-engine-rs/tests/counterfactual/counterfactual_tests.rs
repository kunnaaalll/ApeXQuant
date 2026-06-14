#[cfg(test)]
mod counterfactual_tests {
    use rust_decimal_macros::dec;
    use performance_engine::counterfactual::what_if::CounterfactualResult;
    use performance_engine::counterfactual::alternate_history::{AlternateHistoryEngine, AlternateHistoryContext};
    use performance_engine::counterfactual::parameter_comparison::{ParameterVariant, ParameterComparisonEngine};

    // ─── CounterfactualResult ──────────────────────────────────────────────────

    #[test]
    fn counterfactual_difference_computed_correctly() {
        let r = CounterfactualResult::new(dec!(0.10), dec!(0.18), dec!(0.85), "test".into());
        assert_eq!(r.difference, dec!(0.08));
    }

    #[test]
    fn counterfactual_negative_difference() {
        let r = CounterfactualResult::new(dec!(0.18), dec!(0.10), dec!(0.85), "test".into());
        assert_eq!(r.difference, dec!(-0.08));
    }

    #[test]
    fn counterfactual_equal_outcomes() {
        let r = CounterfactualResult::new(dec!(0.12), dec!(0.12), dec!(0.9), "same".into());
        assert_eq!(r.difference, dec!(0));
    }

    // ─── AlternateHistory ──────────────────────────────────────────────────────

    #[test]
    fn alternate_history_builds_result() {
        let ctx = AlternateHistoryContext {
            session_id: Some("NY".into()),
            regime_id: None,
            symbol_id: None,
            timeframe: None,
            pattern_id: None,
        };
        let r = AlternateHistoryEngine::evaluate(dec!(0.10), dec!(0.15), dec!(0.80), ctx);
        assert_eq!(r.actual_outcome, dec!(0.10));
        assert_eq!(r.alternate_outcome, dec!(0.15));
        assert_eq!(r.difference, dec!(0.05));
    }

    #[test]
    fn alternate_history_deterministic() {
        let ctx = || AlternateHistoryContext {
            session_id: Some("London".into()),
            regime_id: Some("Trend".into()),
            symbol_id: Some("EURUSD".into()),
            timeframe: Some("H1".into()),
            pattern_id: Some("BP".into()),
        };
        for _ in 0..1_000 {
            let r = AlternateHistoryEngine::evaluate(dec!(0.12), dec!(0.20), dec!(0.85), ctx());
            assert_eq!(r.difference, dec!(0.08));
        }
    }

    // ─── ParameterComparison ───────────────────────────────────────────────────

    #[test]
    fn parameter_comparison_best_worst() {
        let variants = vec![
            ParameterVariant { variant_id: "v1".into(), sl: dec!(20), tp: dec!(40), rr: dec!(2.0), filter_score: dec!(0.8), entry_quality: dec!(0.7), outcome: dec!(0.12) },
            ParameterVariant { variant_id: "v2".into(), sl: dec!(15), tp: dec!(45), rr: dec!(3.0), filter_score: dec!(0.9), entry_quality: dec!(0.8), outcome: dec!(0.20) },
            ParameterVariant { variant_id: "v3".into(), sl: dec!(25), tp: dec!(30), rr: dec!(1.2), filter_score: dec!(0.6), entry_quality: dec!(0.5), outcome: dec!(0.04) },
        ];
        let r = ParameterComparisonEngine::compare(variants, dec!(0.85)).unwrap();
        assert_eq!(r.best_variant.variant_id, "v2");
        assert_eq!(r.worst_variant.variant_id, "v3");
        assert_eq!(r.difference, dec!(0.16));
    }

    #[test]
    fn parameter_comparison_requires_two_variants() {
        let variants = vec![
            ParameterVariant { variant_id: "v1".into(), sl: dec!(20), tp: dec!(40), rr: dec!(2.0), filter_score: dec!(0.8), entry_quality: dec!(0.7), outcome: dec!(0.12) },
        ];
        assert!(ParameterComparisonEngine::compare(variants, dec!(0.9)).is_none());
    }

    #[test]
    fn parameter_comparison_deterministic() {
        let make = || vec![
            ParameterVariant { variant_id: "v1".into(), sl: dec!(20), tp: dec!(40), rr: dec!(2.0), filter_score: dec!(0.8), entry_quality: dec!(0.7), outcome: dec!(0.12) },
            ParameterVariant { variant_id: "v2".into(), sl: dec!(15), tp: dec!(45), rr: dec!(3.0), filter_score: dec!(0.9), entry_quality: dec!(0.8), outcome: dec!(0.20) },
        ];
        for _ in 0..1_000 {
            let r = ParameterComparisonEngine::compare(make(), dec!(0.85)).unwrap();
            assert_eq!(r.best_variant.variant_id, "v2");
        }
    }
}
