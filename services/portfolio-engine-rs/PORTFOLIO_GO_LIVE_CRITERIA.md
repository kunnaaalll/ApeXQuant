# APEX V3 PORTFOLIO ENGINE: GO-LIVE CRITERIA

This document defines the absolute gating criteria required to transition the Rust Portfolio Engine V1 from Shadow Mode to full Live Control. 

No system configuration changes or feature requests will override these thresholds.

## 1. Zero Tolerance Metrics
- **0.00% Unhandled Panics:** Any Rust panic within the engine during the 7-day shadow phase constitutes an immediate hard failure.
- **0.00% Unsafe Blocks:** Total absence of `unsafe` Rust.
- **0.00% Data Loss:** The `ShadowStorage` must not drop, truncate, or fail to serialize a single state snapshot.

## 2. Parity Thresholds (Over 7 Consecutive Days)
- **State Agreement:** Must maintain `>99.9%` exact match across all portfolio properties.
- **Recommendation Agreement:** Must maintain `>99.5%` exact match on Increase, Reduce, Close, and Block signals.
- **Health/Quality Drift:** Must remain `<2.0%` aggregate drift over the 7-day rolling window.
- **Max Major Mismatches:** Must be exactly `0` across the 7-day window.

## 3. Performance Thresholds
- **P99 Read/Write Latency:** Must consistently remain `< 20ms`.
- **Memory Consumption:** Must demonstrate a flat baseline over the 7 days (zero leaks).
- **CPU Utilization:** Must operate efficiently within allocated CGroup/container constraints.

## 4. Operational Readiness
- **Complete Runbooks:** Runbooks for Shadow mode recovery, live fallback, and database migration must be fully drafted and peer-reviewed.
- **Alerting Integration:** `ShadowAlert` signals must be verified as successfully routing to the correct PagerDuty/Slack channels.
- **Replay Viability:** A random 1-hour segment of shadow data must be successfully extracted and replayed offline to mathematically identical results.

## 5. Certification Sign-Off
- **CTO Signature:** Required for deployment.
- **Lead Risk Architect Signature:** Required for deployment.
- **Lead Infrastructure Architect Signature:** Required for deployment.

**Current Status:** Pre-Certification (Shadow Mode Active)
