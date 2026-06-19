# Shadow Mode Tests

The shadow mode testing suite rigorously enforces absolute determinism, bounding correctness, and progression logic integrity.

## Test Coverage

- `test_shadow_match_bounds`: Validates bounding and scoring logic.
- `test_drift_percentage_bounds`: Assures relative drift does not exceed 100%.
- `test_match_percentage_bounds`: Verifies match ratios handle zeroes and stay bounded.
- `test_validator_progression`: Checks the staging promotion rule (100 -> 1000 -> 10000).
- `test_forbidden_transitions`: Prevents state skipping and implements immediate step-wise demotion on failure.
- `test_event_rebuild`: Validates structural integrity and equality of serialized events.
- `test_snapshot_restore`: Verifies snapshots can be fully recovered identically.
- `test_report_generation`: Ensures report formatting outputs correct counts and structures.
- `test_determinism_100k_iterations`: Exercises the entire loop 100,000 times ensuring no variance or panics.
