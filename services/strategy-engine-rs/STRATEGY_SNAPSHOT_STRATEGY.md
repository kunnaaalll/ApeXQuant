# Strategy Snapshot Strategy

Snapshots periodically compress streams to bypass unbounded linear processing during startup/recovery.

## Frequencies
The `SnapshotFrequency` dictates pacing:
- `Every10Events`
- `Every50Events`
- `Every100Events`
- `Every500Events`

## Rules
1. A snapshot represents the completely resolved state of the `Aggregatable` entity.
2. Snapshots are stored inside `strategy_snapshots`.
3. If `current_sequence - last_snapshot_sequence >= threshold`, the repository fires a save.
4. Snapshot saves happen concurrently within the same SQL transaction as the triggering event chunk, ensuring no desync.
