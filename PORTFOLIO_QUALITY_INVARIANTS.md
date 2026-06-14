# APEX V3 — Portfolio Quality Invariants

## Core Invariants
- `0.0 <= composite_score <= 100.0` always applies.
- `state` exclusively must be one of `Excellent`, `Good`, `Neutral`, `Weak`, `Critical`.
- `QualitySnapshot` objects are immutable. Every `QualityEvent` application increments the version.
- Decay must strictly monotonically decrease the score or maintain it at `0.0` minimum.
- Negative contributions are prohibited; poor values reflect `0.0` or low positive impact within clamped weights.

## Reliability
- Memory leaks: 0
- Unsafe code blocks: 0
- Panics: 0
- Update latency target: < 1ms Average, < 5ms P99.
