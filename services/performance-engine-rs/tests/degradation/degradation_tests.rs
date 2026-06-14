#[cfg(test)]
mod degradation_tests {
    use rust_decimal_macros::dec;
    use performance_engine::degradation::collapse_detector::{CollapseDetector, CollapseInput, CollapseSignal};
    use performance_engine::degradation::edge_decay::{EdgeDecayEngine, EdgeDecaySnapshot, EdgeDecayState};
    use performance_engine::degradation::strategy_degradation::{
        StrategyDegradationEngine, StrategyDegradationWindow, DegradationState,
    };

    // ─── CollapseDetector ──────────────────────────────────────────────────────

    #[test]
    fn collapse_triggered_on_negative_expectancy() {
        let input = CollapseInput {
            recent_expectancy: dec!(-0.15),
            recent_win_rate: dec!(0.30),
            recent_profit_factor: dec!(0.75),
            consecutive_losses: 12,
            current_drawdown_pct: dec!(0.25),
        };
        let r = CollapseDetector::evaluate(&input);
        assert_eq!(r.signal, CollapseSignal::Triggered);
        assert!(r.severity_score >= dec!(70));
    }

    #[test]
    fn collapse_clear_on_healthy_metrics() {
        let input = CollapseInput {
            recent_expectancy: dec!(0.12),
            recent_win_rate: dec!(0.55),
            recent_profit_factor: dec!(2.0),
            consecutive_losses: 2,
            current_drawdown_pct: dec!(0.05),
        };
        let r = CollapseDetector::evaluate(&input);
        assert_eq!(r.signal, CollapseSignal::Clear);
        assert!(r.severity_score < dec!(25));
    }

    #[test]
    fn collapse_severity_clamped_to_100() {
        let input = CollapseInput {
            recent_expectancy: dec!(-0.50),
            recent_win_rate: dec!(0.10),
            recent_profit_factor: dec!(0.40),
            consecutive_losses: 20,
            current_drawdown_pct: dec!(0.50),
        };
        let r = CollapseDetector::evaluate(&input);
        assert!(r.severity_score <= dec!(100));
    }

    #[test]
    fn collapse_deterministic() {
        let input = CollapseInput {
            recent_expectancy: dec!(-0.08),
            recent_win_rate: dec!(0.40),
            recent_profit_factor: dec!(0.95),
            consecutive_losses: 6,
            current_drawdown_pct: dec!(0.15),
        };
        let r1 = CollapseDetector::evaluate(&input);
        let r2 = CollapseDetector::evaluate(&input);
        assert_eq!(r1.signal, r2.signal);
        assert_eq!(r1.severity_score, r2.severity_score);
    }

    // ─── EdgeDecay ─────────────────────────────────────────────────────────────

    #[test]
    fn edge_decay_collapse_on_large_drop() {
        let snaps = vec![
            EdgeDecaySnapshot { period_label: "P1".into(), edge_score: dec!(0.90) },
            EdgeDecaySnapshot { period_label: "P2".into(), edge_score: dec!(0.50) },
            EdgeDecaySnapshot { period_label: "P3".into(), edge_score: dec!(0.15) },
        ];
        let r = EdgeDecayEngine::evaluate(&snaps).unwrap();
        assert_eq!(r.state, EdgeDecayState::Collapse);
    }

    #[test]
    fn edge_decay_healthy_on_flat() {
        let snaps = vec![
            EdgeDecaySnapshot { period_label: "P1".into(), edge_score: dec!(0.80) },
            EdgeDecaySnapshot { period_label: "P2".into(), edge_score: dec!(0.79) },
            EdgeDecaySnapshot { period_label: "P3".into(), edge_score: dec!(0.78) },
        ];
        let r = EdgeDecayEngine::evaluate(&snaps).unwrap();
        assert_eq!(r.state, EdgeDecayState::Healthy);
    }

    #[test]
    fn edge_decay_needs_at_least_two() {
        let snaps = vec![EdgeDecaySnapshot { period_label: "P1".into(), edge_score: dec!(0.80) }];
        assert!(EdgeDecayEngine::evaluate(&snaps).is_none());
    }

    // ─── StrategyDegradation ───────────────────────────────────────────────────

    #[test]
    fn strategy_degradation_collapse() {
        let windows = vec![
            StrategyDegradationWindow { period_label: "P1".into(), trade_count: 50, expectancy: dec!(0.12), profit_factor: dec!(2.0), confidence: dec!(0.9) },
            StrategyDegradationWindow { period_label: "P2".into(), trade_count: 50, expectancy: dec!(-0.08), profit_factor: dec!(0.75), confidence: dec!(0.5) },
        ];
        let r = StrategyDegradationEngine::evaluate(&windows).unwrap();
        assert_eq!(r.state, DegradationState::Collapse);
    }

    #[test]
    fn strategy_degradation_healthy() {
        let windows = vec![
            StrategyDegradationWindow { period_label: "P1".into(), trade_count: 100, expectancy: dec!(0.12), profit_factor: dec!(2.0), confidence: dec!(0.9) },
            StrategyDegradationWindow { period_label: "P2".into(), trade_count: 100, expectancy: dec!(0.11), profit_factor: dec!(1.95), confidence: dec!(0.88) },
        ];
        let r = StrategyDegradationEngine::evaluate(&windows).unwrap();
        assert_eq!(r.state, DegradationState::Healthy);
    }

    #[test]
    fn strategy_degradation_needs_at_least_two() {
        let windows = vec![
            StrategyDegradationWindow { period_label: "P1".into(), trade_count: 50, expectancy: dec!(0.12), profit_factor: dec!(2.0), confidence: dec!(0.9) },
        ];
        assert!(StrategyDegradationEngine::evaluate(&windows).is_none());
    }
}
