# Strategy Foundation Invariants

- **Determinism**: Precision must not drift over time.
- **Safety**: `unsafe` is strictly forbidden at the crate level.
- **Bounds**: `HealthScore` and `ConfidenceScore` must never fall outside [0, 100].
- **State Changes**: State cannot transition from `Retired` to `Active`.
