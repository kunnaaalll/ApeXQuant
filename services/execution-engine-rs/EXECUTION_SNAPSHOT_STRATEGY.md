# Execution Snapshot Strategy

Snapshots accelerate reconstruction times by providing serialized baselines of `ExecutionState`.

## Snapshot Frequencies
- `Every10Events`
- `Every50Events`
- `Every100Events`
- `Every500Events`
- `Every1000Events`

No magical strings or numbers are permitted in frequency configuration; values must map to `SnapshotFrequency` enum cases cleanly.
