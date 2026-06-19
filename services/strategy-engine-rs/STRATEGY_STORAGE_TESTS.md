# Strategy Storage Tests

Test structure handles logic independently from live PostgreSQL servers to allow robust offline CI.

## Unit Tests
- `test_serializer_roundtrip`: Verifies `serde_json` maps events precisely both ways.
- `test_deterministic_serialization`: Ensures order preservation.
- `test_determinism_100k_iterations`: Stress test over 100k iterations ensuring exact serialized string matches.

## Integration Tests
Marked `#[ignore]` and located in `tests/storage_tests.rs`:
- `test_event_append_and_load`
- `test_snapshot_append_and_load`
- `test_rebuild_from_events`
- `test_snapshot_acceleration`
- `test_repository_transaction`
- `test_concurrent_append`
- `test_atomic_commit`
- `test_snapshot_restore`

To run DB tests, specify a `DATABASE_URL` and use `cargo test -- --ignored`.
