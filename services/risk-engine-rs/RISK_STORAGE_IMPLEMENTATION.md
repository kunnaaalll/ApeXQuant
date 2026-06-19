# Risk Engine Storage Implementation

## Overview
The Event Sourcing Storage Engine is the permanent memory layer for the `risk-engine-rs` service. It is designed to provide immutable event sourcing, deterministic replayability, and transactional persistence via PostgreSQL. It handles persistence strictly without housing any business logic.

## Modules
- **`events`**: Defines the `EventRecord` struct and the `PortfolioEventWrapper` enum, ensuring all risk engine events are serializable, immutable, and fully typed.
- **`snapshots`**: Defines the `SnapshotRecord` struct and `SnapshotFrequency` policy, determining when the risk engine should checkpoint state.
- **`rebuilder`**: Provides a generic `RiskEventRebuilder` capable of applying events in a purely functional, deterministic manner.
- **`pg_store`**: Handles raw interactions with PostgreSQL utilizing `sqlx`. Manages optimistic locking and payload serialization.
- **`repository`**: Provides the top-level orchestration layer for appending events with atomicity and loading streams.

## Principles
1. **Zero Panics, Unwraps, Expects**: Every error is surfaced as an explicit `Result`.
2. **Zero Unsafe**: Pure Rust, strictly memory safe.
3. **No Business Logic**: The storage layer relies entirely on traits and external domain functions for event applications and reconstructions.
