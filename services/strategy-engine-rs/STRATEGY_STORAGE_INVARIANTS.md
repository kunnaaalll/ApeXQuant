# Strategy Storage Invariants

The event sourcing engine honors the following architectural invariants:

1. **Replay Equivalence**: 
   `Replay(stream)` must perfectly equal `Replay(snapshot + tail events)` for every boundary.
2. **Deterministic Serialization**: 
   Events serialize equivalently regardless of runtime OS/Architecture.
3. **Sequence Monotonicity**: 
   Sequences strictly monotonically increase per aggregate ID. 
4. **All-or-Nothing Transactions**:
   A batch containing 50 events either writes all 50 or 0. If a snapshot threshold is triggered during this save, the snapshot is included inside the same SQL transaction.
5. **No Blind Unwrapping**:
   Storage loading logic guarantees graceful escalation of any corruption via nested error types.
