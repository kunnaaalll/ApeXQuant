#![allow(warnings, clippy::all, deprecated)]
use crate::learning::{EvidenceAccumulator, LearningAssessment};
use rust_decimal::Decimal;

#[test]
fn test_evidence_accumulator() {
    let mut acc = EvidenceAccumulator::new();

    // EMA initially 0.0
    assert_eq!(acc.assess(), LearningAssessment::Stable);

    // Strengthen edge history EMA
    for _ in 0..20 {
        acc.record_event(
            true,
            Decimal::new(5, 1),
            Decimal::new(80, 0),
            Decimal::new(3, 1),
        );
    }

    assert!(acc.edge_history_ema > Decimal::new(2, 1));
    assert_eq!(acc.assess(), LearningAssessment::Strengthening);
}
