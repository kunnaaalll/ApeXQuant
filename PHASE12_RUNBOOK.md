# APEX V3 — Phase 12: MT5 Demo Validation & Operational Qualification Runbook

**Version:** 1.0  
**Status:** Pre-Demo Validation  

## Objective
Qualify APEX V3 for live MT5 Demo trading through institutional-grade operational validation. Treat the MT5 Demo environment exactly as if it were managing live institutional capital. Success is measured by Correctness, Determinism, Stability, Recovery, Reconciliation, and Reliability.

## Prerequisites
- MetaTrader 5 Terminal running on the host machine and logged into the intended Demo account ($5,000 starting balance).
- `mt5_bridge.py` running in a Windows/Wine environment or successfully connecting to the MT5 Terminal via the Python `MetaTrader5` SDK.
- APEX V3 core services must be buildable via `docker-compose`.

## Environment Configuration
The environment variables for Phase 12 are defined in `infrastructure/docker/.env.phase12`.
All validation data is persisted in a fresh set of PostgreSQL tables created via `infrastructure/docker/init-scripts/001_apex_v3_schema.sql`.

## Execution Stages

The `scripts/phase12/run_phase12.sh` script automates the sequential execution of the following stages:

### Stage 1: Infrastructure Preflight Check (`00_preflight_check.sh`)
- Probes all 20+ services (ports and HTTP health checks).
- Ensures MT5 bridge is reachable and the terminal is connected with trade permissions allowed.
- Verifies database connectivity.

### Stage 2: Market Data Validation (`01_market_data_validator.sh`)
- Runs for a configurable duration (default 24h).
- Monitors tick ingestion for 4 instruments (EURUSD, GBPUSD, XAUUSD, US30).
- Validates monotonicity of timestamps (no retrograde ticks).
- Verifies tick and candle persistence in PostgreSQL.
- Checks OHLC parity.

### Stage 3: Execution Pipeline Validation (`02_execution_validator.sh`)
- Submits market, limit, and stop orders across multiple instruments.
- Modifies pending order stop-losses and take-profits.
- Cancels pending orders.
- Executes partial and full closes of open positions.
- **CRITICAL:** Verifies that fill events are broker-confirmed and not synthetic.

### Stage 4: Recovery Testing (`03_recovery_tester.sh`)
- Opens multiple live positions.
- Sequentially restarts each core engine (Execution, Risk, Portfolio, etc.), the Event Bus, PostgreSQL, and Redis.
- Verifies that zero positions are orphaned and no duplicate orders are created upon recovery.

### Stage 5: Chaos Testing (`04_chaos_injector_phase12.sh`)
- Randomly injects failures (`SIGKILL` to engines, `docker restart` to infrastructure, `SIGSTOP`/`SIGCONT` to Redis) while the system is under load.
- Validates that the system recovers automatically with zero data loss.

### Stage 6: Replay Validation (`05_replay_validator.sh`)
- Checkpoints the database state (portfolio, positions, risk, events hashes).
- Triggers the ReplayEngine to rebuild state from the event stream.
- Verifies that the replayed state exactly matches the original deterministic state.

### Stage 7: Broker Reconciliation (`06_broker_reconciler.sh`)
- Runs as a background daemon during the campaign.
- Compares engine state vs broker state every hour (or configurable interval).
- Detects positional drift and logs metrics to the database.

### Stage 8: 1000-Trade Campaign (`07_trade_campaign.sh`)
- Monitors the system while it autonomously executes 1,000+ trades.
- Validates execution throughput and stability under high-frequency conditions.
- Fails immediately if duplicate executions are detected.

## Reporting & Artifacts
All stage reports and logs are generated in the `phase12_reports/` directory (or the path defined by `PHASE12_REPORT_DIR`).
- `DAILY_VALIDATION_REPORT.md` (Stage 2)
- `RECOVERY_VALIDATION_REPORT.md` (Stage 4)
- `REPLAY_CERTIFICATION_REPORT.md` (Stage 6)
- `BROKER_RECONCILIATION_REPORT.md` (Stage 7)

## Grafana Dashboard
A comprehensive dashboard is available at `infrastructure/monitoring/grafana/dashboards/phase12_validation.json` to monitor execution throughput, latency, system errors, and reconciliation drift in real-time.
