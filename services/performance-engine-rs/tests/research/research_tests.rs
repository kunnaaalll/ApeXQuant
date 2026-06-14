#[cfg(test)]
mod research_tests {
    use rust_decimal_macros::dec;
    use performance_engine::research::opportunity_ranking::{OpportunityRanking, OpportunityRankingEngine};
    use performance_engine::research::edge_ranking::{EdgeRanking, EdgeRankingEngine};
    use performance_engine::research::weakness_ranking::{WeaknessRanking, WeaknessRankingEngine, WeaknessState};

    fn make_opportunity(label: &str, edge: f64, confidence: f64, sample: f64) -> OpportunityRanking {
        OpportunityRanking {
            rank: 0,
            dimension: "symbol".into(),
            label: label.to_string(),
            edge: dec!(edge),
            confidence: dec!(confidence),
            sample_quality: dec!(sample),
            historical_evidence: 100,
            explanation: String::new(),
        }
    }

    fn make_weakness(label: &str, expectancy: f64, pf: f64, dd: f64) -> WeaknessRanking {
        WeaknessRanking {
            rank: 0,
            dimension: "symbol".into(),
            label: label.to_string(),
            expectancy: dec!(expectancy),
            profit_factor: dec!(pf),
            max_drawdown: dec!(dd),
            confidence: dec!(0.8),
            trade_count: 50,
            state: WeaknessState::Watchlist,
            explanation: String::new(),
        }
    }

    // ─── OpportunityRanking ────────────────────────────────────────────────────

    #[test]
    fn opportunity_best_ranked_first() {
        let candidates = vec![
            make_opportunity("EURUSD", 0.10, 0.8, 1.0),
            make_opportunity("GBPUSD", 0.20, 0.9, 1.0), // best
            make_opportunity("USDJPY", 0.05, 0.6, 0.8),
        ];
        let ranked = OpportunityRankingEngine::rank(candidates);
        assert_eq!(ranked[0].label, "GBPUSD");
        assert_eq!(ranked[0].rank, 1);
        assert_eq!(ranked.last().unwrap().rank as usize, ranked.len());
    }

    #[test]
    fn opportunity_rank_is_contiguous_from_one() {
        let candidates = vec![
            make_opportunity("A", 0.12, 0.8, 1.0),
            make_opportunity("B", 0.08, 0.7, 0.9),
            make_opportunity("C", 0.15, 0.9, 1.0),
        ];
        let ranked = OpportunityRankingEngine::rank(candidates);
        let ranks: Vec<u32> = ranked.iter().map(|r| r.rank).collect();
        assert_eq!(ranks, vec![1, 2, 3]);
    }

    #[test]
    fn opportunity_tiebreak_is_lexicographic() {
        // Same score — alphabetical label wins
        let candidates = vec![
            make_opportunity("ZZZ", 0.10, 1.0, 1.0),
            make_opportunity("AAA", 0.10, 1.0, 1.0),
        ];
        let ranked = OpportunityRankingEngine::rank(candidates);
        assert_eq!(ranked[0].label, "AAA");
    }

    #[test]
    fn opportunity_ranking_deterministic() {
        let candidates = || vec![
            make_opportunity("EURUSD", 0.10, 0.8, 1.0),
            make_opportunity("GBPUSD", 0.20, 0.9, 1.0),
            make_opportunity("USDJPY", 0.05, 0.6, 0.8),
        ];
        let r1 = OpportunityRankingEngine::rank(candidates());
        let r2 = OpportunityRankingEngine::rank(candidates());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.label, b.label);
            assert_eq!(a.rank, b.rank);
        }
    }

    // ─── EdgeRanking ───────────────────────────────────────────────────────────

    #[test]
    fn edge_ranking_best_expectancy_wins() {
        let candidates = vec![
            EdgeRanking { rank: 0, strategy_id: "S1".into(), dimension: "pattern".into(), label: "Breakout".into(), expectancy: dec!(0.15), profit_factor: dec!(2.5), sharpe_approx: dec!(1.2), confidence: dec!(0.9), trade_count: 200, explanation: String::new() },
            EdgeRanking { rank: 0, strategy_id: "S1".into(), dimension: "pattern".into(), label: "Reversal".into(), expectancy: dec!(0.08), profit_factor: dec!(1.8), sharpe_approx: dec!(0.9), confidence: dec!(0.8), trade_count: 150, explanation: String::new() },
        ];
        let ranked = EdgeRankingEngine::rank(candidates);
        assert_eq!(ranked[0].label, "Breakout");
    }

    // ─── WeaknessRanking ───────────────────────────────────────────────────────

    #[test]
    fn weakness_worst_ranked_first() {
        let candidates = vec![
            make_weakness("AUDUSD", -0.12, 0.7, 0.25), // worst
            make_weakness("NZDUSD", 0.02, 0.95, 0.08),
            make_weakness("EURGBP", -0.03, 0.85, 0.15),
        ];
        let ranked = WeaknessRankingEngine::rank(candidates);
        assert_eq!(ranked[0].label, "AUDUSD");
        assert_eq!(ranked[0].rank, 1);
    }

    #[test]
    fn weakness_forbidden_assigned_on_severe_metrics() {
        let candidates = vec![make_weakness("BAD", -0.15, 0.60, 0.35)];
        let ranked = WeaknessRankingEngine::rank(candidates);
        assert_eq!(ranked[0].state, WeaknessState::Forbidden);
    }

    #[test]
    fn weakness_watchlist_on_borderline() {
        let candidates = vec![make_weakness("OK", 0.05, 1.1, 0.05)];
        let ranked = WeaknessRankingEngine::rank(candidates);
        assert_eq!(ranked[0].state, WeaknessState::Watchlist);
    }
}
