#![allow(clippy::module_inception)]

#[cfg(test)]
mod tests {
    use crate::correlation::*;
    use proptest::prelude::*;
    use rust_decimal::Decimal;

    proptest! {
        #[test]
        fn test_matrix_bounds(
            corr in -200..200i32
        ) {
            let mut matrix = CorrelationMatrix::new();
            let raw_decimal = Decimal::new(corr as i64, 2); // Ranges roughly -2.00 to 2.00

            matrix.set_correlation("Symbol", "AAPL", "MSFT", raw_decimal);
            let retrieved = matrix.get_correlation("Symbol", "AAPL", "MSFT");

            // Should be bounded [-1, 1]
            let pos_one = Decimal::new(1, 0);
            let neg_one = Decimal::new(-1, 0);
            assert!(retrieved >= neg_one);
            assert!(retrieved <= pos_one);
        }

        #[test]
        fn test_matrix_symmetry(
            entity_a in "[A-Z]{3,4}",
            entity_b in "[A-Z]{3,4}",
            corr in -100..100i32
        ) {
            let mut matrix = CorrelationMatrix::new();
            let decimal_corr = Decimal::new(corr as i64, 2);

            matrix.set_correlation("Theme", &entity_a, &entity_b, decimal_corr);

            let a_to_b = matrix.get_correlation("Theme", &entity_a, &entity_b);
            let b_to_a = matrix.get_correlation("Theme", &entity_b, &entity_a);

            assert_eq!(a_to_b, b_to_a);
        }
    }

    #[test]
    fn test_diagonal_equals_one() {
        let matrix = CorrelationMatrix::new();
        assert_eq!(
            matrix.get_correlation("Sector", "Tech", "Tech"),
            Decimal::new(1, 0)
        );
    }

    #[test]
    fn test_hidden_leverage_calculations() {
        let hla = HiddenLeverageAssessment::new(
            Decimal::new(10, 1), // 1.0
            Decimal::new(5, 1),  // 0.5
            Decimal::new(2, 1),  // 0.2
            Decimal::new(0, 0),
            Decimal::new(0, 0),
        );

        // 1.0 * 1.5 + 0.5 * 1.2 + 0.2 * 1.0 = 1.5 + 0.6 + 0.2 = 2.3
        assert_eq!(hla.total_hidden_leverage_ratio, Decimal::new(23, 1));
        assert_eq!(hla.state, HiddenLeverageState::High);
    }

    #[test]
    fn test_cluster_formation_and_clamping() {
        let cluster = CorrelationCluster::new(
            CorrelationCategory::RiskOnTech,
            vec!["AAPL".into(), "MSFT".into()],
            Decimal::new(10, 0),
            Decimal::new(150, 0), // Over 100
        );

        assert_eq!(cluster.concentration, Decimal::new(100, 0));
        assert_eq!(cluster.severity, CorrelationSeverity::Critical);
    }

    #[test]
    fn test_100k_deterministic_iterations() {
        let mut matrix = CorrelationMatrix::new();
        let base_corr = Decimal::new(5, 1); // 0.5

        for i in 0..100_000 {
            let a = format!("SYM_{}", i % 100);
            let b = format!("SYM_{}", (i + 1) % 100);
            let factor = Decimal::new((i % 10) as i64, 1);
            let corr = (base_corr * factor).min(Decimal::new(1, 0));
            matrix.set_correlation("Symbol", &a, &b, corr);
        }

        // Must be exactly deterministic across runs
        let test_val = matrix.get_correlation("Symbol", "SYM_10", "SYM_11");
        assert_eq!(test_val, Decimal::new(0, 0));
        // Let's just check bounds and deterministic completion without panics
        assert!(test_val >= Decimal::new(-1, 0));
        assert!(test_val <= Decimal::new(1, 0));
    }

    #[test]
    fn test_snapshot_replay() {
        let matrix = CorrelationMatrix::new();
        let hla = HiddenLeverageAssessment::new(
            Decimal::new(0, 0),
            Decimal::new(0, 0),
            Decimal::new(0, 0),
            Decimal::new(0, 0),
            Decimal::new(0, 0),
        );
        let cluster = CorrelationCluster::new(
            CorrelationCategory::RiskOffBonds,
            vec!["TLT".into()],
            Decimal::new(10, 0),
            Decimal::new(50, 0),
        );

        let snapshot = CorrelationRiskSnapshot::new(
            1,
            1670000000,
            CorrelationRiskEvent::ClusterDetected {
                cluster: cluster.clone(),
            },
            matrix.clone(),
            hla.state,
            vec![cluster],
        );

        assert_eq!(snapshot.version, 1);
        assert_eq!(snapshot.clusters.len(), 1);
    }
}
