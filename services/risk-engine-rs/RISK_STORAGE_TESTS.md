# Risk Storage Tests

The `tests.rs` module ensures that the storage engine conforms to strict correctness guarantees.

## Logic Tests
- `test_rebuild_from_events`: Verifies that a generic state aggregator can sequentially apply events.
- `test_snapshot_plus_events`: Proves that `rebuild(snapshot + partial stream) == rebuild(full stream)`.
- `test_ordering`: Validates sequential numbering.
- `test_determinism_100k`: Subjects the rebuilder to 100,000 iterations over a large event stream to prove zero memory corruption, panics, or state drift.

## Database Tests
The database integration tests require an active PostgreSQL connection (`DATABASE_URL`). These tests simulate complete database lifecycles:
- `test_event_append_and_load`: Inserts and retrieves an event payload via `pg_store`.
- `test_snapshot_append_and_load`: Ensures `JSONB` snapshots can round-trip through PostgreSQL correctly.
- `test_concurrent_append`: Verifies optimistic locking behaviors under load.

All integration tests are currently ignored in the default suite to allow pure compilation verification, but will execute fully in CI environments with Postgres services enabled.
