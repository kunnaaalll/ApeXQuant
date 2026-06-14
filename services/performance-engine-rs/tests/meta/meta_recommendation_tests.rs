#[cfg(test)]
mod meta_recommendation_tests {
    use rust_decimal_macros::dec;
    use performance_engine::meta::meta_recommendation::{
        MetaAction, MetaRecommendationEngine, MetaRecommendationInput,
    };

    fn base_input(name: &str) -> MetaRecommendationInput {
        MetaRecommendationInput {
            strategy_name: name.to_string(),
            health_score: 75,
            expectancy: dec!(0.12),
            profit_factor: dec!(2.0),
            win_rate: dec!(0.55),
            max_drawdown: dec!(0.06),
            confidence: dec!(0.80),
            stability: dec!(0.80),
            trade_count: 200,
            expectancy_drift: dec!(0),
            oos_ratio: Some(dec!(0.90)),
            overfit_penalty: dec!(0.95),
        }
    }

    // ─── Action routing ────────────────────────────────────────────────────────

    #[test]
    fn recommends_retire_on_collapse() {
        let mut input = base_input("Dying");
        input.health_score = 0;
        input.expectancy = dec!(-0.20);
        input.profit_factor = dec!(0.60);
        let r = MetaRecommendationEngine::recommend(&input);
        assert_eq!(r.action, MetaAction::Retire);
    }

    #[test]
    fn recommends_pause_on_severe_drawdown() {
        let mut input = base_input("Stressed");
        input.max_drawdown = dec!(0.25);
        input.health_score = 15;
        let r = MetaRecommendationEngine::recommend(&input);
        assert_eq!(r.action, MetaAction::Pause);
    }

    #[test]
    fn recommends_pause_on_overfit_risk() {
        let mut input = base_input("Overfit");
        input.overfit_penalty = dec!(0.40);
        let r = MetaRecommendationEngine::recommend(&input);
        assert_eq!(r.action, MetaAction::Pause);
    }

    #[test]
    fn recommends_reduce_on_drifting_edge() {
        let mut input = base_input("Drifting");
        input.expectancy_drift = dec!(0.12); // > 8% threshold
        let r = MetaRecommendationEngine::recommend(&input);
        assert_eq!(r.action, MetaAction::Reduce);
    }

    #[test]
    fn recommends_research_on_small_sample() {
        let mut input = base_input("New");
        input.trade_count = 20;
        let r = MetaRecommendationEngine::recommend(&input);
        assert_eq!(r.action, MetaAction::Research);
    }

    #[test]
    fn recommends_research_on_low_confidence() {
        let mut input = base_input("Uncertain");
        input.confidence = dec!(0.30);
        let r = MetaRecommendationEngine::recommend(&input);
        assert_eq!(r.action, MetaAction::Research);
    }

    #[test]
    fn recommends_increase_on_elite_strategy() {
        let input = MetaRecommendationInput {
            strategy_name: "Elite".to_string(),
            health_score: 90,
            expectancy: dec!(0.20),
            profit_factor: dec!(2.5),
            win_rate: dec!(0.65),
            max_drawdown: dec!(0.04),
            confidence: dec!(0.95),
            stability: dec!(0.90),
            trade_count: 500,
            expectancy_drift: dec!(-0.01), // slightly positive — improving
            oos_ratio: Some(dec!(0.95)),
            overfit_penalty: dec!(0.95),
        };
        let r = MetaRecommendationEngine::recommend(&input);
        assert_eq!(r.action, MetaAction::IncreaseAllocation);
    }

    #[test]
    fn recommends_continue_for_normal_strategy() {
        let input = base_input("Normal");
        let r = MetaRecommendationEngine::recommend(&input);
        assert_eq!(r.action, MetaAction::Continue);
    }

    // ─── Output integrity ──────────────────────────────────────────────────────

    #[test]
    fn recommendation_has_non_empty_reason() {
        let input = base_input("Any");
        let r = MetaRecommendationEngine::recommend(&input);
        assert!(!r.reason.is_empty());
        assert!(!r.largest_contributor.is_empty());
        assert!(!r.largest_weakness.is_empty());
    }

    #[test]
    fn recommendation_confidence_in_range() {
        let input = base_input("Any");
        let r = MetaRecommendationEngine::recommend(&input);
        assert!(r.confidence >= dec!(0) && r.confidence <= dec!(1));
    }

    // ─── Determinism (10,000 passes) ──────────────────────────────────────────

    #[test]
    fn recommendation_deterministic_10k() {
        let input = base_input("Det");
        let reference = MetaRecommendationEngine::recommend(&input);
        for _ in 0..10_000 {
            let r = MetaRecommendationEngine::recommend(&input);
            assert_eq!(r.action, reference.action);
            assert_eq!(r.reason, reference.reason);
        }
    }

    // ─── Invariant: confidence is always ≤ 1.0 ────────────────────────────────

    #[test]
    fn confidence_never_exceeds_one() {
        let variants: Vec<MetaRecommendationInput> = vec![
            base_input("A"),
            { let mut i = base_input("B"); i.health_score = 0; i.expectancy = dec!(-0.5); i },
            { let mut i = base_input("C"); i.overfit_penalty = dec!(0.30); i },
            { let mut i = base_input("D"); i.trade_count = 5; i },
        ];
        for v in &variants {
            let r = MetaRecommendationEngine::recommend(v);
            assert!(r.confidence <= dec!(1), "confidence exceeded 1.0 for action {:?}", r.action);
        }
    }
}
