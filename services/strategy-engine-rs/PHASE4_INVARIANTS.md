# Phase 4 Institutional Invariants

This system is strictly bound by institutional validation rules:

1. **Absolute Determinism**: `rust_decimal` used exclusively. No `f32` or `f64` in Phase 4 code. Replayable on any architecture.
2. **Zero Unsafe Code**: Library is rooted with `#![deny(unsafe_code)]`.
3. **Zero Panics**: No `.unwrap()` or `.expect()` calls left unchecked without strict guaranteed prior safety logic.
4. **Zero Randomness**: Pure functional state changes.
5. **Zero Machine Learning**: Neural nets explicitly banned.
6. **Bounded Memory**: Fixed-size structs, `HashMap` and `VecDeque` explicitly restricted, avoiding indefinite heap allocations.
7. **Event Sourcing Ready**: Dedicated `events.rs` and `snapshot.rs` files included for state rebuilding and ledger integration.
