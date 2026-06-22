# Execution Storage Tests

The `src/storage/tests.rs` module strictly verifies the deterministic capabilities of the storage engine. 

## Tests Implemented
- **test_event_ordering**: Ensures sequential IDs are accepted.
- **test_sequence_violation**: Verifies an event `1 -> 3` fails correctly.
- **test_snapshot_restore**: Guarantees perfectly matching payloads after a snapshot is applied.
- **test_event_rebuild**: Validates full event stream reconstruction matches runtime.
- **test_repository_roundtrip**: End-to-end load/save testing.
- **test_version_monotonicity**: Prohibits retrograde version shifts.
- **test_transaction_atomicity**: Verifies aborted commits leave no partial traces.
- **test_determinism_100k_iterations**: Benchmarks zero corruption over a vast stress cycle.
