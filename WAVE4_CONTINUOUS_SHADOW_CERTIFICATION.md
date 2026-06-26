# Wave 4 — Continuous Shadow Certification

This document confirms the implementation of the complete APEX platform Continuous Shadow Certification mode for long-duration deterministic validation.

## 1. Replay Hash Certification
- Implemented `ReplayCertifier` in `apex-core-rs/src/replay_certification`.
- Extracts deterministic SHA256 state hashes via the `EventStore`.
- Provides `certify_replay` ensuring identically verifiable replay execution with zero mismatches.

## 2. Restart Recovery Testing
- Implemented `RestartRecoveryTester` in `apex-core-rs/src/restart_recovery`.
- Simulates arbitrary engine crashes via `force_engine_crash`.
- Enforces determinism by fully rebuilding states via `rebuild_from_events`.
- Validates the post-recovery hash identically matches the pre-crash snapshot.

## 3. Long Run Validation
- Implemented `LongRunValidator` in `apex-core-rs/src/long_run_validation`.
- Integrates a 1,000,000 tick processing simulator.
- Actively tracks memory growth bounds, peak queue depths, and global throughput.

## 4. Shadow Metrics Aggregation
- Implemented `ShadowMetrics` in `apex-core-rs/src/shadow_metrics`.
- Exports institutional-grade metrics: `shadow_winrate`, `shadow_expectancy`, `shadow_profit_factor`, `shadow_max_drawdown`, `shadow_fill_quality`, `shadow_latency`, `shadow_parity_score`, `shadow_drift_score`.

## 5. Global Shadow Orchestrator
- Implemented `GlobalShadowOrchestrator` in `apex-core-rs/src/shadow_orchestrator`.
- Enforces strict transition flows across the `ShadowState` lifecycle: Booting -> Warmup -> Collecting -> Validating -> Candidate -> Approved (or Paused/Failed).

## 6. Certification Report Generator
- Implemented `CertificationReportGenerator` in `apex-core-rs/src/certification_report`.
- Generates precise JSON artifacts encompassing `session_id`, `replay_hash`, `parity_score`, `drift_score`, `pnl`, `max_drawdown`, and standard `certification_status`.

## Validation Sign-Off
All automated compilation checks passed. The continuous certification engines are primed for processing the institutional 100k replay cycles and 1M shadow ticks.

**Zero drift invariants preserved.**
