# APEX V3 Portfolio Storage Engine Tests

This document outlines the rigorous testing strategy for the Portfolio Storage Engine. Testing must validate determinism, data integrity, concurrency limits, and strict invariants.

## Test Suites

### 1. Unit Tests
- **Serialization/Deserialization:** Verify that `PortfolioEventWrapper` and `PortfolioSnapshotWrapper` can correctly serialize to and from JSON without loss of precision or missing fields.
- **Record Generation:** Validate that `EventRecord::new()` and `SnapshotRecord::new()` properly attach UUIDs and UTC timestamps.
- **EventRebuilder Logic:** Test `EventRebuilder::rebuild` using a mocked state and a simple addition/subtraction event structure to verify sequential application logic.

### 2. Integration Tests
- **Database Initialization:** Ensure `PgPool` connects to a test database and runs required schema migrations cleanly.
- **Event Appending:** Validate that `repository.save_event_with_snapshot()` correctly persists an event.
- **Snapshot Retrieval:** Write a snapshot and retrieve it using `repository.load_latest_snapshot()`, ensuring the payload matches exactly.
- **Atomicity Checks:** Force a failure during the snapshot insertion phase of `save_event_with_snapshot()` and verify the event insertion rolls back (transaction safety).

### 3. Replay Tests
- **Deterministic Replay:** Generate a synthetic sequence of 100 events. Apply them to an initial state using `EventRebuilder`. Serialize the final state. Reload the 100 events from the DB, run them through the rebuilder again, and assert that the resulting serialized state is byte-for-byte identical.

### 4. Concurrency & Stress Tests
- **Optimistic Locking Conflicts:** Spawn 10 concurrent tokio tasks attempting to append an event with `version = 5` to the same aggregate. Assert that exactly 1 succeeds and 9 fail with a database constraint error.
- **High Throughput Appends:** Spawn 1,000 asynchronous tasks writing sequential events to the same aggregate. Verify that all 1,000 events are persisted, and the max version equals 1,000.

### 5. Property-Based Tests (Proptest)
- **Fuzzing Payloads:** Generate completely random JSON structures (fuzzing) as payloads and ensure the database layer never panics, but rather safely rejects invalid types if schema constraints apply, or securely stores them as JSONB.

## Run Instructions

To run the storage test suite:

```bash
# Start a local postgres test container
docker-compose up -d postgres-test

# Run the test suite
cargo test --package portfolio-engine-rs --test storage_tests
```
