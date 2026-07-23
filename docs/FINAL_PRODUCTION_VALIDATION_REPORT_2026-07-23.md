# APEX V3.1 Final Production Validation Report

Date: 2026-07-23  
Scope: repository audit, Learning/Event Bus integration, AI availability, and executable checks

## Verdict

❌ **Production Certification Failed**

The repository cannot be production-certified from this environment. The required runtime stack was not available: `docker` is not installed, so PostgreSQL, Redis, Event Bus, broker/MT5 bridge, service health endpoints, historical replay, trace propagation, chaos recovery, and performance measurements could not be exercised. Per the brief, those items are **NOT VERIFIED**, not assumed successful.

## Changes made during validation

- Removed the Learning Engine's direct Redis Pub/Sub dependency and subscription loop.
- Added Event Bus gRPC subscription with a consumer group, latest-start position, batch processing, and acknowledgements.
- Subscribed Learning to the relevant trade/position/risk/performance/portfolio topics present in the current contracts; only `PositionClosed` payloads are processed today. Other subscribed payloads are acknowledged and logged as unsupported.
- Removed the Event Bus's runtime requirement for NATS, which was absent from `infrastructure/docker/docker-compose.yml` and not used by the active gRPC publish/subscribe path. Redis remains used by Event Bus sequencing.
- Changed the AI inference endpoint to return HTTP 503 with `MODEL_UNAVAILABLE` when no production model is present; removed deterministic price-delta output presented as inference.
- Changed default AI health from false `OK` to explicit `MODEL_UNAVAILABLE` / `NOT_CONNECTED` statuses.

## Executed evidence

| Check | Result | Evidence |
|---|---|---|
| `cargo check -p learning-engine-rs -p event-bus-rs` | PASS | Completed successfully after Event Bus contract correction |
| `cargo test -p learning-engine-rs -p event-bus-rs --no-fail-fast` | PASS | 16 Learning tests passed; Event Bus has 0 tests; no failures |
| `cargo check -p ai-engine-rs -p learning-engine-rs -p event-bus-rs` | PASS | Completed successfully after AI availability change |
| Docker/runtime stack | NOT VERIFIED | `docker: command not found` |
| Postgres/Redis/NATS/Event Bus connectivity | NOT VERIFIED | No container runtime available |
| Broker/MT5 connectivity | NOT VERIFIED | No runtime and no demo-broker execution |
| Historical replay with real data | NOT VERIFIED | No executable replay run with real historical data |
| End-to-end trace IDs and event counts | NOT VERIFIED | No runtime trace evidence |
| Failure/chaos tests | NOT VERIFIED | No live dependencies to restart/disconnect |
| CPU/memory/latency/throughput benchmarks | NOT VERIFIED | No production process run |
| TLS/auth/authz/certificate validation | NOT VERIFIED | No deployed endpoint/configuration to exercise |

## Remaining production blockers

1. The AI Engine still does not start its gRPC/API inference server from `main.rs`, does not connect to PostgreSQL/Redis/Event Bus, and reports no live dependency state. It is therefore not available for the production pipeline.
2. Learning's current event handling does not yet persist the requested features, labels, datasets, model metadata, and training history from all six required event classes. It handles closed-position PnL updates only; this is **PARTIAL**.
3. Learning derives `strategy_id` from correlation baggage, but the current closed-position protobuf has no direct strategy field. Runtime event-contract verification is required to prove this is populated and correct.
4. Event Bus has legacy NATS health/publisher modules and an unused async-NATS dependency even though the active server path no longer connects to NATS. This is static legacy code and should be removed or explicitly restored as the chosen transport before certification.
5. Orchestrator is a GraphQL status surface over channels; its workflows are declarative status objects rather than verified coordinating workflows. Startup ordering, readiness aggregation, graceful shutdown, and gateway routing remain **NOT VERIFIED**.
6. Compose health/port mappings and every service's health/readiness/metrics/version/dependency contract were not runtime-tested.

## Component status

| Component | Status |
|---|---|
| Market Data | ⚠ Partial — source/tests exist; runtime feed and real-data replay NOT VERIFIED |
| Signal | ⚠ Partial — code/tests exist; runtime signal path NOT VERIFIED |
| Strategy | ⚠ Partial — code exists; runtime gRPC/event path NOT VERIFIED |
| Risk | ⚠ Partial — code exists; runtime broker/database/event dependencies NOT VERIFIED |
| Execution | ⚠ Partial — code exists; MT5/demo execution NOT VERIFIED |
| Position | ⚠ Partial — code exists; runtime event consumption NOT VERIFIED |
| Portfolio | ⚠ Partial — code exists; runtime consistency NOT VERIFIED |
| Performance | ⚠ Partial — calculations/tests exist; runtime recording NOT VERIFIED |
| Learning | ⚠ Partial — Event Bus integration compiles and tests pass; full required persistence NOT VERIFIED |
| AI | ❌ Failed for production use — no model artifact and no live inference server |
| Analytics | ⚠ Partial — code exists; runtime updates NOT VERIFIED |
| Event Bus | ⚠ Partial — gRPC/storage code compiles; live reliability, persistence, backpressure, ordering, and stress NOT VERIFIED |
| Orchestrator | ❌ Failed for certification — coordination behavior NOT VERIFIED |

## Required next evidence

Run the stack on a host with Docker and real historical data/demo-broker access, then capture health/readiness/metrics responses, Event Bus publish/subscribe acknowledgements, trace IDs for each pipeline stage, replay counts, database rows for each persisted artifact, chaos recovery results, and benchmark output. Until those artifacts exist, the only valid final verdict is **Production Certification Failed**.
