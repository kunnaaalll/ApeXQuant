#![allow(warnings, clippy::all, deprecated)]
use crate::memory::{ConfidenceMemory, EdgeMemory};
use rust_decimal::Decimal;

#[test]
fn test_confidence_memory_bounds() {
    let mut mem = ConfidenceMemory::new(3);

    mem.record(Decimal::new(80, 0), Decimal::new(0, 0), Decimal::new(5, 1));
    mem.record(Decimal::new(85, 0), Decimal::new(0, 0), Decimal::new(6, 1));
    mem.record(Decimal::new(90, 0), Decimal::new(0, 0), Decimal::new(7, 1));

    assert_eq!(mem.historical_confidence.len(), 3);
    assert_eq!(
        mem.historical_confidence.front(),
        Some(&Decimal::new(90, 0))
    );
    assert_eq!(mem.historical_confidence.back(), Some(&Decimal::new(80, 0)));

    // Should eject oldest
    mem.record(Decimal::new(95, 0), Decimal::new(0, 0), Decimal::new(8, 1));
    assert_eq!(mem.historical_confidence.len(), 3);
    assert_eq!(
        mem.historical_confidence.front(),
        Some(&Decimal::new(95, 0))
    );
    assert_eq!(mem.historical_confidence.back(), Some(&Decimal::new(85, 0)));
}

#[test]
fn test_edge_memory_bounds() {
    let mut mem = EdgeMemory::new(2);

    mem.record(Decimal::new(5, 1), Decimal::new(2, 1));
    mem.record(Decimal::new(6, 1), Decimal::new(3, 1));
    assert_eq!(mem.rolling_edge_history.len(), 2);

    mem.record(Decimal::new(7, 1), Decimal::new(4, 1));
    assert_eq!(mem.rolling_edge_history.len(), 2);
    assert_eq!(mem.rolling_edge_history.front(), Some(&Decimal::new(7, 1)));
    assert_eq!(mem.rolling_edge_history.back(), Some(&Decimal::new(6, 1)));
}
