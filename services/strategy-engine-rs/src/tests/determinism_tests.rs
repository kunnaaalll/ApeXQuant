#![allow(warnings, clippy::all, deprecated)]
use crate::learning::EvidenceAccumulator;
use rust_decimal::Decimal;

#[test]
fn test_determinism_100k_iterations() {
    let mut acc = EvidenceAccumulator::new();
    let edge_input = Decimal::new(25, 2); // 0.25
    let conf_input = Decimal::new(80, 0); // 80
    let exp_input = Decimal::new(10, 2); // 0.10

    for _ in 0..100_000 {
        acc.record_event(true, exp_input, conf_input, edge_input);
    }

    // Since we use strict decimal operations, there should be no panic, and results must not be NaN or diverge.
    // Given the EMA approaches the input value:
    assert_eq!(acc.edge_history_ema.round_dp(4), edge_input.round_dp(4));
    assert_eq!(
        acc.confidence_history_ema.round_dp(4),
        conf_input.round_dp(4)
    );
    assert_eq!(
        acc.expectancy_history_ema.round_dp(4),
        exp_input.round_dp(4)
    );
}
