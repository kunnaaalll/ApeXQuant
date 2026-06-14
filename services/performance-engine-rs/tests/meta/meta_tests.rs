#[cfg(test)]
mod meta_tests {
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    use performance_engine::meta::strategy_registry::StrategyProfile;
    use performance_engine::meta::strategy_state::StrategyState;
    use performance_engine::meta::strategy_evolution::{StrategyEvolutionAssessment, EvolutionState};
    use performance_engine::meta::strategy_comparison::StrategyComparisonEngine;
    use performance_engine::meta::strategy_health::{StrategyHealth, HealthState};

    fn make_profile(name: &str, expectancy: f64, confidence: f64, stability: f64, drawdown: f64) -> StrategyProfile {
        let mut p = StrategyProfile::new(Uuid::new_v4(), name.to_string());
        p.expectancy = dec!(expectancy);
        p.confidence = dec!(confidence);
        p.stability = dec!(stability);
        p.max_drawdown = dec!(drawdown);
        p
    }

    // ─── StrategyHealth tests ─────────────────────────────────────────────────

    #[test]
    fn health_classify_states() {
        assert_eq!(StrategyHealth::from_score(95).state, HealthState::Excellent);
        assert_eq!(StrategyHealth::from_score(75).state, HealthState::Healthy);
        assert_eq!(StrategyHealth::from_score(55).state, HealthState::Normal);
        assert_eq!(StrategyHealth::from_score(35).state, HealthState::Weak);
        assert_eq!(StrategyHealth::from_score(15).state, HealthState::Critical);
        assert_eq!(StrategyHealth::from_score(0).state, HealthState::Dead);
    }

    #[test]
    fn health_collapse_is_immediate() {
        let h = StrategyHealth::from_score(90);
        let collapsed = h.transition(0);
        assert_eq!(collapsed.score, 0);
        assert!(collapsed.is_collapsed);
    }

    #[test]
    fn health_recovery_is_gradual() {
        let h = StrategyHealth::from_score(50);
        let recovered = h.transition(100);
        // Should only recover MAX_RECOVERY_STEP (5) points per cycle
        assert_eq!(recovered.score, 55);
        assert!(!recovered.is_collapsed);
    }

    #[test]
    fn health_clamped_to_100() {
        let h = StrategyHealth::from_score(98);
        let next = h.transition(100);
        assert!(next.score <= 100);
    }

    #[test]
    fn health_synthesise_dead_on_negative_expectancy() {
        let score = StrategyHealth::synthesise(
            dec!(0.5),
            dec!(-0.15), // expectancy well below collapse threshold
            dec!(1.5),
            dec!(0.05),
            dec!(0.8),
            dec!(0.8),
        );
        assert_eq!(score, 0);
    }

    #[test]
    fn health_synthesise_excellent_on_strong_metrics() {
        let score = StrategyHealth::synthesise(
            dec!(0.65),   // win rate
            dec!(0.15),   // expectancy
            dec!(2.5),    // profit factor
            dec!(0.03),   // max drawdown
            dec!(0.9),    // confidence
            dec!(0.9),    // stability
        );
        assert!(score >= 80, "Score was {}", score);
    }

    #[test]
    fn health_synthesise_deterministic() {
        let score_a = StrategyHealth::synthesise(
            dec!(0.55), dec!(0.10), dec!(2.0), dec!(0.05), dec!(0.8), dec!(0.8),
        );
        let score_b = StrategyHealth::synthesise(
            dec!(0.55), dec!(0.10), dec!(2.0), dec!(0.05), dec!(0.8), dec!(0.8),
        );
        assert_eq!(score_a, score_b);
    }

    // ─── StrategyEvolution tests ───────────────────────────────────────────────

    #[test]
    fn evolution_improving() {
        let assessment = StrategyEvolutionAssessment::new(
            dec!(0.10),   // expectancy improved
            dec!(-0.02),  // drawdown improved
            dec!(0.01),
            dec!(0.05),
        );
        assert_eq!(assessment.state, EvolutionState::Improving);
    }

    #[test]
    fn evolution_collapsing_on_negative_expectancy() {
        let assessment = StrategyEvolutionAssessment::new(
            dec!(-0.25), // breaches threshold
            dec!(0.05),
            dec!(-0.05),
            dec!(-0.10),
        );
        assert_eq!(assessment.state, EvolutionState::Collapsing);
    }

    #[test]
    fn evolution_weakening() {
        let assessment = StrategyEvolutionAssessment::new(
            dec!(-0.07), // below weakening threshold
            dec!(0.04),
            dec!(0),
            dec!(0),
        );
        assert_eq!(assessment.state, EvolutionState::Weakening);
    }

    // ─── StrategyComparison tests ──────────────────────────────────────────────

    #[test]
    fn comparison_winner_has_highest_score() {
        let profiles = vec![
            make_profile("A", 0.10, 0.8, 0.7, 0.05),
            make_profile("B", 0.20, 0.9, 0.9, 0.03), // best
            make_profile("C", 0.05, 0.6, 0.5, 0.10),
        ];
        let result = StrategyComparisonEngine::compare(profiles).unwrap();
        assert_eq!(result.winner.name, "B");
        assert!(result.runner_up.is_some());
        assert!(result.loser.is_some());
    }

    #[test]
    fn comparison_single_profile_no_runner_up() {
        let profiles = vec![make_profile("A", 0.10, 0.8, 0.7, 0.05)];
        let result = StrategyComparisonEngine::compare(profiles).unwrap();
        assert_eq!(result.winner.name, "A");
        assert!(result.runner_up.is_none());
        assert!(result.loser.is_none());
    }

    #[test]
    fn comparison_empty_returns_none() {
        let result = StrategyComparisonEngine::compare(vec![]);
        assert!(result.is_none());
    }

    // ─── Monte Carlo: StrategyHealth determinism over 10,000 runs ─────────────

    #[test]
    fn health_monte_carlo_determinism() {
        for _ in 0..10_000 {
            let score = StrategyHealth::synthesise(
                dec!(0.55), dec!(0.12), dec!(1.8), dec!(0.06), dec!(0.75), dec!(0.70),
            );
            assert_eq!(score, StrategyHealth::synthesise(
                dec!(0.55), dec!(0.12), dec!(1.8), dec!(0.06), dec!(0.75), dec!(0.70),
            ));
        }
    }
}
