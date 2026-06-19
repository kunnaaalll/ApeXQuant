# Shadow Mode Implementation

## Overview
Phase 9 introduces the institutional-grade shadow execution environment for the APEX V3 Strategy Engine. It enables deterministic, state-isolated evaluation of new strategies against historical reference strategies without mutating production state.

## Architecture

- **Comparison Engine**: Bounded evaluation of strategy properties (health, confidence, edge, drift, allocation, etc.).
- **Drift Engine**: Calculates absolute and relative deviations (0-100 clamped).
- **Statistics Engine**: Tracks performance distribution (matches, mismatches, warnings).
- **Go-Live Validator**: Staged promotion (`NotReady` -> `Monitoring` -> `Candidate` -> `Approved`) with immediate step-wise demotion to prevent unearned state skips.
- **Event & Snapshot Support**: Designed for perfect replayability without randomness.

## Module Layout
- `src/shadow/comparison.rs`
- `src/shadow/drift.rs`
- `src/shadow/statistics.rs`
- `src/shadow/reporter.rs`
- `src/shadow/validator.rs`
- `src/shadow/events.rs`
- `src/shadow/snapshot.rs`
- `tests/shadow_tests.rs`
