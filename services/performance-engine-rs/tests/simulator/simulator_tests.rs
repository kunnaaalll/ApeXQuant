#[cfg(test)]
mod simulator_tests {
    use rust_decimal_macros::dec;
    use performance_engine::simulator::replay_engine::{
        ReplayEngine, ReplayFilter, TradeRecord,
    };
    use performance_engine::simulator::variant_runner::{Variant, VariantRunner};
    use performance_engine::simulator::configuration_evaluator::ConfigurationEvaluator;

    fn win(id: &str, session: &str, symbol: &str, regime: &str, rr: f64) -> TradeRecord {
        TradeRecord {
            trade_id: id.to_string(),
            session: session.to_string(),
            regime: regime.to_string(),
            symbol: symbol.to_string(),
            timeframe: "H1".to_string(),
            pattern_id: "BP".to_string(),
            sl: dec!(20),
            tp: dec!(40),
            rr: dec!(rr),
            entry_quality: dec!(0.8),
            r_outcome: dec!(1.0),
            is_win: true,
        }
    }

    fn loss(id: &str, session: &str, symbol: &str, regime: &str, rr: f64) -> TradeRecord {
        TradeRecord {
            trade_id: id.to_string(),
            session: session.to_string(),
            regime: regime.to_string(),
            symbol: symbol.to_string(),
            timeframe: "H1".to_string(),
            pattern_id: "BP".to_string(),
            sl: dec!(20),
            tp: dec!(40),
            rr: dec!(rr),
            entry_quality: dec!(0.8),
            r_outcome: dec!(-1.0),
            is_win: false,
        }
    }

    // ─── ReplayEngine ──────────────────────────────────────────────────────────

    #[test]
    fn replay_no_filter_computes_correctly() {
        let trades = vec![
            win("t1", "London", "EURUSD", "Trend", 2.0),
            win("t2", "London", "EURUSD", "Trend", 2.0),
            loss("t3", "London", "EURUSD", "Trend", 2.0),
            loss("t4", "London", "EURUSD", "Trend", 2.0),
        ];
        let result = ReplayEngine::replay(&trades, &ReplayFilter::default());
        assert_eq!(result.trade_count, 4);
        assert_eq!(result.win_rate, dec!(0.5));
        assert_eq!(result.expectancy, dec!(0)); // 2 wins +1, 2 losses -1 = 0
    }

    #[test]
    fn replay_session_filter_isolates_trades() {
        let trades = vec![
            win("t1", "London", "EURUSD", "Trend", 2.0),
            win("t2", "NY", "EURUSD", "Trend", 2.0),
            loss("t3", "NY", "EURUSD", "Trend", 2.0),
        ];
        let filter = ReplayFilter { session: Some("NY".to_string()), ..Default::default() };
        let result = ReplayEngine::replay(&trades, &filter);
        assert_eq!(result.trade_count, 2);
        assert_eq!(result.win_rate, dec!(0.5));
    }

    #[test]
    fn replay_empty_set_returns_empty() {
        let result = ReplayEngine::replay(&[], &ReplayFilter::default());
        assert_eq!(result.trade_count, 0);
        assert_eq!(result.expectancy, dec!(0));
    }

    #[test]
    fn replay_deterministic_repeated_calls() {
        let trades: Vec<TradeRecord> = (0..20)
            .map(|i| if i % 2 == 0 { win(&i.to_string(), "London", "EURUSD", "Trend", 2.0) }
                 else { loss(&i.to_string(), "London", "EURUSD", "Trend", 2.0) })
            .collect();
        let r1 = ReplayEngine::replay(&trades, &ReplayFilter::default());
        let r2 = ReplayEngine::replay(&trades, &ReplayFilter::default());
        assert_eq!(r1.expectancy, r2.expectancy);
        assert_eq!(r1.win_rate, r2.win_rate);
        assert_eq!(r1.max_drawdown, r2.max_drawdown);
    }

    #[test]
    fn replay_max_drawdown_computed() {
        // 2 wins then 3 losses
        let trades = vec![
            win("t1", "L", "E", "T", 2.0),
            win("t2", "L", "E", "T", 2.0),
            loss("t3", "L", "E", "T", 2.0),
            loss("t4", "L", "E", "T", 2.0),
            loss("t5", "L", "E", "T", 2.0),
        ];
        let r = ReplayEngine::replay(&trades, &ReplayFilter::default());
        assert!(r.max_drawdown > dec!(0), "Expected non-zero drawdown");
    }

    // ─── VariantRunner ─────────────────────────────────────────────────────────

    #[test]
    fn variant_runner_returns_same_count_as_variants() {
        let trades = vec![
            win("t1", "London", "EURUSD", "Trend", 2.0),
            loss("t2", "London", "GBPUSD", "Range", 2.0),
        ];
        let variants = vec![
            Variant { variant_id: "v1".into(), description: "London".into(), filter: ReplayFilter { session: Some("London".to_string()), ..Default::default() } },
            Variant { variant_id: "v2".into(), description: "All".into(), filter: ReplayFilter::default() },
        ];
        let results = VariantRunner::run_all(&trades, &variants);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].variant_id, "v1");
    }

    // ─── ConfigurationEvaluator ────────────────────────────────────────────────

    #[test]
    fn config_evaluator_picks_best_variant() {
        let trades: Vec<TradeRecord> = (0..10)
            .map(|i| if i < 7 { win(&i.to_string(), "London", "EURUSD", "Trend", 2.0) }
                 else { loss(&i.to_string(), "London", "EURUSD", "Trend", 2.0) })
            .collect();
        let variants = vec![
            Variant { variant_id: "v_all".into(), description: "all trades".into(), filter: ReplayFilter::default() },
            Variant { variant_id: "v_rr2".into(), description: "rr >= 2".into(), filter: ReplayFilter { min_rr: Some(dec!(2)), ..Default::default() } },
        ];
        let eval = ConfigurationEvaluator::evaluate(&trades, variants).unwrap();
        assert!(!eval.best_variant_id.is_empty());
        assert!(eval.best_expectancy >= eval.worst_expectancy);
    }

    #[test]
    fn config_evaluator_empty_returns_none() {
        let result = ConfigurationEvaluator::evaluate(&[], vec![]);
        assert!(result.is_none());
    }

    // ─── Monte Carlo: Replay determinism (100,000 iterations) ─────────────────

    #[test]
    fn replay_monte_carlo_determinism_100k() {
        let trades: Vec<TradeRecord> = (0..100)
            .map(|i| if i % 3 != 0 {
                win(&i.to_string(), "London", "EURUSD", "Trend", 2.0)
            } else {
                loss(&i.to_string(), "London", "EURUSD", "Trend", 2.0)
            })
            .collect();
        let filter = ReplayFilter::default();
        let reference = ReplayEngine::replay(&trades, &filter);

        for _ in 0..100_000 {
            let result = ReplayEngine::replay(&trades, &filter);
            assert_eq!(result.expectancy, reference.expectancy);
            assert_eq!(result.win_rate, reference.win_rate);
            assert_eq!(result.profit_factor, reference.profit_factor);
        }
    }
}
