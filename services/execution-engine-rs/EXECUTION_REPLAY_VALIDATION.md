# Execution Replay Validation

Replay Validation ensures total deterministic playback of the execution engine.
Given a `ValidationSnapshot` and an array of historical `ValidationEvent`s, the engine guarantees that the rebuilt state perfectly matches the original state.
Replay Status:
- `Exact`: Perfect match.
- `Mismatch`: State divergence detected.
- `Corrupted`: Snapshot or events unreadable.
