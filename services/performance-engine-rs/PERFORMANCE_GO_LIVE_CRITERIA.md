# Go-Live Certification Criteria

To be moved out of Shadow Mode and officially deployed to Production, the Rust `performance-engine-rs` MUST meet the following criteria to earn the `InstitutionalCertified` state.

## 1. Zero Panics under Stress
The system must endure 10M concurrent synthetic evaluations injected through the `StressSuite` without yielding a single thread panic, deadlock, or OOM event.

## 2. 99.9% Parity Agreement
Parity with the legacy TypeScript Engine MUST strictly exceed 99.9% across Expectancy, Confidence, and Meta-recommendation outputs, tested over the last 30 days of live event data.

## 3. Strict Determinism
Given seed `X`, running 100k consecutive cycles MUST yield an output hash `Y` that never drifts. Any non-determinism violates core constraints.

## 4. Replay Equality
Snapshot regeneration from events MUST match sequential states perfectly.

## 5. Latency & Throughput
Latency must average `< 2ms` with a p99 `< 10ms`. Throughput must exceed `100,000 evt/s` per core.
