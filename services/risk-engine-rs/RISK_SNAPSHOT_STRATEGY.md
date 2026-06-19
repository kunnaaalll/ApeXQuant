# Risk Snapshot Strategy

## Why Snapshots?
As an event stream grows (often into the millions of events per portfolio), rebuilding the state from sequence 0 becomes computationally intensive. Snapshots act as "save points", eliminating the need to process the entire history.

## Snapshot Frequency
The engine supports configurable frequencies (`SnapshotFrequency`):
- `Every10Events`
- `Every50Events`
- `Every100Events`
- `Manual`

## Reconstruction Workflow
1. Load the most recent `SnapshotRecord`.
2. Determine the `version` of the snapshot.
3. Fetch all `EventRecord` entries where `sequence > snapshot.version`.
4. Fast-forward the snapshot state by sequentially applying the retrieved events.

## Persistence
Snapshots are stored in the `risk_snapshots` table as `JSONB` payloads. They are completely immutable and must never be altered retroactively.
