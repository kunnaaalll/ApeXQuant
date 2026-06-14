# APEX V3 Portfolio Storage Engine Implementation

## Event Sourcing Philosophy

The APEX V3 Portfolio Storage Engine acts as the permanent, immutable memory of the entire portfolio ecosystem. It is designed around Event Sourcing principles:

1. **State as a Derivative:** The current state of the portfolio (or any of its sub-engines like Exposure, Heat, Allocation) is a derivative of a sequence of events.
2. **Append-Only History:** Events are strictly append-only. There are no `UPDATE` or `DELETE` operations on event logs.
3. **Auditability & Replayability:** Every transition is recorded. This allows institutional audit teams to replay ten years of portfolio history deterministically and exactly recreate the portfolio state at any given microsecond.
4. **No Hidden Mutations:** A state cannot change without a corresponding event being published and persisted.

## Architecture

The storage layer is primarily backed by **PostgreSQL**, interacting asynchronously via `sqlx`.

### Key Components

1. **`pg_store::PostgresPortfolioStore`**: The low-level database abstraction layer. It manages connection pooling, asynchronous CRUD, and SQL statement execution.
2. **`repository::PortfolioRepository`**: The high-level API. It provides domain-specific methods (like `save_event_with_snapshot`) and manages transactional atomicity.
3. **`events::EventRecord`**: The schema representing a persisted event.
4. **`snapshots::SnapshotRecord`**: The schema representing an optimized state at a specific point in time.

## Reconstruction Guarantees

The Engine guarantees that `rebuild_from_events(initial_state, events_1_to_N)` will perfectly yield `State_N`. There is absolutely no divergence permitted. To achieve this:

- Floating-point non-determinism is eliminated by using exact decimal arithmetic (`rust_decimal`) throughout the system.
- Snapshot serialization and deserialization are verified to be perfectly reversible.

## Snapshots

While events are the source of truth, reading 1,000,000 events to reconstruct the current state is slow. The system uses **Snapshots** to optimize read paths.

- Snapshots are retained at multiple frequencies: `Realtime`, `M1`, `M5`, `M15`, `H1`, `D1`, `Weekly`, `Monthly`, and `Historical`.
- A replay operation finds the nearest snapshot prior to the target time and replays the events from that snapshot forward.

## Transaction Behavior

Writing to the storage engine must be perfectly atomic:
- When a state transition occurs, the event (and potentially a snapshot) must be saved in the **same database transaction**.
- Optimistic locking is used. If two concurrent transitions attempt to save an event with `version = N`, the database will enforce a unique constraint on `(aggregate_id, version)`, causing the second transaction to abort and roll back.

## Failure Scenarios & Edge Cases

- **Database Disconnects:** Handled natively by the `sqlx` connection pool which automatically attempts to reconnect. The transaction is aborted and the domain layer must retry the operation.
- **Concurrent Writes:** Handled by optimistic locking (`version` sequence).
- **Corrupted Snapshots:** A snapshot is considered corrupted if deserialization fails. The system can self-heal by falling back to a previous snapshot and replaying a longer chain of events.

## Known Limitations

- PostgreSQL is vertically scalable, but if event volume exceeds hundreds of thousands of events per second (highly unlikely for a portfolio engine, which usually operates on a lower frequency than a tick execution engine), partitioning the `portfolio_events` table by time or aggregate ID will be required.
