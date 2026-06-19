# Strategy Storage Implementation

The permanent memory layer for `strategy-engine-rs` has been implemented following strict principles:
- **No panic!, unwrap(), or expect()**: Complete and total reliance on `Result`.
- **Exclusively `rust_decimal::Decimal`**: Avoided `f32/f64` arithmetic.
- **Transactional Guarantees**: Handled via `sqlx::Transaction`.

## Storage Mod Layout

- `events.rs`: Defines `EventRecord` and the exhaustive `StrategyEventWrapper`.
- `snapshots.rs`: Defines `SnapshotRecord` and `SnapshotFrequency`.
- `rebuilder.rs`: Defines `StrategyEventRebuilder` and `Aggregatable` trait.
- `repository.rs`: Main interface `StrategyRepository` for atomic saves.
- `pg_store.rs`: Low-level DB interop via `sqlx` without SQL string interpolations.
- `serializer.rs`: Robust JSON serializer mapping to `Result`.

This adheres strictly to Phase 7 requirements for the Strategy Engine.
