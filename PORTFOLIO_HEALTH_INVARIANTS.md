# APEX V3 — Portfolio Health Invariants

## Core Invariants
- `0 <= composite_score <= 100` always applies (represented by Rust `u8` clamped strictly to 100 maximum).
- `state` exclusively must be one of `Excellent`, `Healthy`, `Normal`, `Weak`, `Critical`.
- `HealthSnapshot` objects are immutable and strictly versioned.
- Health recovery progression is monotonically increasing but restricted per recovery tick (a smooth curve, no immediate `Critical` to `Healthy` jumps).
- Negative contributions are structurally prevented by design.

## Reliability
- No panics under extreme numeric inputs (saturating math applied).
- Race conditions eliminated through strict deterministic ownership.
- Update latency target: < 1ms Average, < 5ms P99.
