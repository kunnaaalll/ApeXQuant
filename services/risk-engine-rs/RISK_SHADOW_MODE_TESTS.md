# Shadow Mode Tests

## Determinism
`test_determinism` executes 100,000 iterations to ensure zero variation in logic, zero `panic!` invocations, and precise decimal calculations over time.

## Symmetry and Bounds
- `test_comparison_symmetry` asserts that identical states strictly yield `ComparisonState::ExactMatch`.
- `test_drift_bounds` guarantees absolute limits on divergence calculations, preventing `NaN` and `Infinity`.
- `test_statistics_agreement_bounds` verifies that agreement percentages never exceed the [0, 100] range.

## Sequential Validation
- `test_validator_transitions` enforces sequential flow and denies arbitrary state leaps (e.g. `Rejected` -> `Approved`).
