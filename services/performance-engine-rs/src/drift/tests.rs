use crate::drift::*;
#[cfg(test)]
use rust_decimal_macros::dec;

#[test]
fn test_edge_drift_critical() {
    let drift = EdgeDrift::evaluate(dec!(0.1), dec!(0.15), dec!(0.5));
    assert_eq!(drift.state, DriftState::Critical);
}

#[test]
fn test_determinism_drift_loop() {
    let mut prev: Option<EdgeDrift> = None;
    for _ in 0..100_000 {
        let drift = EdgeDrift::evaluate(dec!(0.4), dec!(0.38), dec!(0.35));
        if let Some(p) = &prev {
            assert_eq!(drift.percentage_change, p.percentage_change);
        }
        prev = Some(drift);
    }
}
