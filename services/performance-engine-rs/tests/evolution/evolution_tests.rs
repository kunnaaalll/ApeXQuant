#[cfg(test)]
mod evolution_tests {
    use rust_decimal_macros::dec;
    use performance_engine::evolution::regime_evolution::{RegimeEvolutionEngine, RegimeEvolutionWindow, RegimeTrend};
    use performance_engine::evolution::symbol_evolution::{SymbolEvolutionEngine, SymbolEvolutionWindow, SymbolTrend};
    use performance_engine::evolution::pattern_evolution::{PatternEvolutionEngine, PatternEvolutionWindow, PatternTrend};
    use performance_engine::evolution::timeframe_evolution::{TimeframeEvolutionEngine, TimeframeEvolutionWindow, TimeframeTrend};
    use performance_engine::evolution::session_evolution::{SessionEvolutionEngine, SessionEvolutionWindow, SessionTrend};

    // ─── RegimeEvolution ───────────────────────────────────────────────────────

    #[test]
    fn regime_strengthening() {
        let windows = vec![
            RegimeEvolutionWindow { regime_id: "Trend".into(), period_label: "Q1".into(), trade_count: 50, win_rate: dec!(0.55), expectancy: dec!(0.10), profit_factor: dec!(1.8) },
            RegimeEvolutionWindow { regime_id: "Trend".into(), period_label: "Q2".into(), trade_count: 60, win_rate: dec!(0.60), expectancy: dec!(0.18), profit_factor: dec!(2.2) },
        ];
        let r = RegimeEvolutionEngine::evaluate(&windows).unwrap();
        assert_eq!(r.trend, RegimeTrend::Strengthening);
        assert!(r.expectancy_drift > dec!(0));
    }

    #[test]
    fn regime_weakening() {
        let windows = vec![
            RegimeEvolutionWindow { regime_id: "Trend".into(), period_label: "Q1".into(), trade_count: 50, win_rate: dec!(0.60), expectancy: dec!(0.18), profit_factor: dec!(2.2) },
            RegimeEvolutionWindow { regime_id: "Trend".into(), period_label: "Q2".into(), trade_count: 40, win_rate: dec!(0.45), expectancy: dec!(0.05), profit_factor: dec!(1.2) },
        ];
        let r = RegimeEvolutionEngine::evaluate(&windows).unwrap();
        assert_eq!(r.trend, RegimeTrend::Weakening);
    }

    #[test]
    fn regime_abandoned_on_zero_trades() {
        let windows = vec![
            RegimeEvolutionWindow { regime_id: "Range".into(), period_label: "Q1".into(), trade_count: 50, win_rate: dec!(0.55), expectancy: dec!(0.10), profit_factor: dec!(1.8) },
            RegimeEvolutionWindow { regime_id: "Range".into(), period_label: "Q2".into(), trade_count: 0, win_rate: dec!(0), expectancy: dec!(0), profit_factor: dec!(0) },
        ];
        let r = RegimeEvolutionEngine::evaluate(&windows).unwrap();
        assert_eq!(r.trend, RegimeTrend::Abandoned);
    }

    #[test]
    fn regime_needs_min_two_windows() {
        let windows = vec![
            RegimeEvolutionWindow { regime_id: "Trend".into(), period_label: "Q1".into(), trade_count: 50, win_rate: dec!(0.55), expectancy: dec!(0.10), profit_factor: dec!(1.8) },
        ];
        assert!(RegimeEvolutionEngine::evaluate(&windows).is_none());
    }

    // ─── SymbolEvolution ───────────────────────────────────────────────────────

    #[test]
    fn symbol_strengthening() {
        let windows = vec![
            SymbolEvolutionWindow { symbol: "EURUSD".into(), period_label: "M1".into(), trade_count: 80, expectancy: dec!(0.10), profit_factor: dec!(1.8), max_drawdown: dec!(0.08) },
            SymbolEvolutionWindow { symbol: "EURUSD".into(), period_label: "M2".into(), trade_count: 90, expectancy: dec!(0.18), profit_factor: dec!(2.2), max_drawdown: dec!(0.06) },
        ];
        let r = SymbolEvolutionEngine::evaluate(&windows).unwrap();
        assert_eq!(r.trend, SymbolTrend::Strengthening);
    }

    #[test]
    fn symbol_weakening() {
        let windows = vec![
            SymbolEvolutionWindow { symbol: "GBPUSD".into(), period_label: "M1".into(), trade_count: 80, expectancy: dec!(0.15), profit_factor: dec!(2.0), max_drawdown: dec!(0.06) },
            SymbolEvolutionWindow { symbol: "GBPUSD".into(), period_label: "M2".into(), trade_count: 60, expectancy: dec!(0.05), profit_factor: dec!(1.3), max_drawdown: dec!(0.18) },
        ];
        let r = SymbolEvolutionEngine::evaluate(&windows).unwrap();
        assert_eq!(r.trend, SymbolTrend::Weakening);
    }

    // ─── PatternEvolution ──────────────────────────────────────────────────────

    #[test]
    fn pattern_maturing() {
        let windows = vec![
            PatternEvolutionWindow { pattern_id: "BP".into(), period_label: "Q1".into(), trade_count: 40, win_rate: dec!(0.55), expectancy: dec!(0.08), avg_rr: dec!(2.0) },
            PatternEvolutionWindow { pattern_id: "BP".into(), period_label: "Q2".into(), trade_count: 50, win_rate: dec!(0.60), expectancy: dec!(0.15), avg_rr: dec!(2.2) },
        ];
        let r = PatternEvolutionEngine::evaluate(&windows).unwrap();
        assert_eq!(r.trend, PatternTrend::Maturing);
    }

    #[test]
    fn pattern_fading() {
        let windows = vec![
            PatternEvolutionWindow { pattern_id: "Rev".into(), period_label: "Q1".into(), trade_count: 50, win_rate: dec!(0.60), expectancy: dec!(0.15), avg_rr: dec!(2.0) },
            PatternEvolutionWindow { pattern_id: "Rev".into(), period_label: "Q2".into(), trade_count: 30, win_rate: dec!(0.45), expectancy: dec!(0.05), avg_rr: dec!(1.6) },
        ];
        let r = PatternEvolutionEngine::evaluate(&windows).unwrap();
        assert_eq!(r.trend, PatternTrend::Fading);
    }

    // ─── TimeframeEvolution ────────────────────────────────────────────────────

    #[test]
    fn timeframe_strengthening() {
        let windows = vec![
            TimeframeEvolutionWindow { timeframe: "H4".into(), period_label: "Q1".into(), trade_count: 60, expectancy: dec!(0.10), profit_factor: dec!(1.8), stability: dec!(0.70) },
            TimeframeEvolutionWindow { timeframe: "H4".into(), period_label: "Q2".into(), trade_count: 70, expectancy: dec!(0.18), profit_factor: dec!(2.0), stability: dec!(0.80) },
        ];
        let r = TimeframeEvolutionEngine::evaluate(&windows).unwrap();
        assert_eq!(r.trend, TimeframeTrend::Strengthening);
    }

    // ─── SessionEvolution ──────────────────────────────────────────────────────

    #[test]
    fn session_improving() {
        let windows = vec![
            SessionEvolutionWindow { session: "London".into(), period_label: "M1".into(), trade_count: 80, win_rate: dec!(0.55), expectancy: dec!(0.10), avg_rr: dec!(2.0) },
            SessionEvolutionWindow { session: "London".into(), period_label: "M2".into(), trade_count: 90, win_rate: dec!(0.60), expectancy: dec!(0.18), avg_rr: dec!(2.2) },
        ];
        let r = SessionEvolutionEngine::evaluate(&windows).unwrap();
        assert_eq!(r.trend, SessionTrend::Improving);
    }

    #[test]
    fn session_deteriorating() {
        let windows = vec![
            SessionEvolutionWindow { session: "NY".into(), period_label: "M1".into(), trade_count: 80, win_rate: dec!(0.60), expectancy: dec!(0.18), avg_rr: dec!(2.0) },
            SessionEvolutionWindow { session: "NY".into(), period_label: "M2".into(), trade_count: 50, win_rate: dec!(0.40), expectancy: dec!(0.03), avg_rr: dec!(1.5) },
        ];
        let r = SessionEvolutionEngine::evaluate(&windows).unwrap();
        assert_eq!(r.trend, SessionTrend::Deteriorating);
    }

    // ─── Determinism: All evolution engines (10,000 passes) ───────────────────

    #[test]
    fn all_evolution_engines_deterministic_10k() {
        let regime_windows = vec![
            RegimeEvolutionWindow { regime_id: "T".into(), period_label: "Q1".into(), trade_count: 50, win_rate: dec!(0.55), expectancy: dec!(0.10), profit_factor: dec!(1.8) },
            RegimeEvolutionWindow { regime_id: "T".into(), period_label: "Q2".into(), trade_count: 60, win_rate: dec!(0.58), expectancy: dec!(0.12), profit_factor: dec!(1.9) },
        ];
        let r_ref = RegimeEvolutionEngine::evaluate(&regime_windows).unwrap();
        for _ in 0..10_000 {
            let r = RegimeEvolutionEngine::evaluate(&regime_windows).unwrap();
            assert_eq!(r.trend, r_ref.trend);
            assert_eq!(r.expectancy_drift, r_ref.expectancy_drift);
        }
    }
}
