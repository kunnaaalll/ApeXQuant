# Portfolio State Implementation

## Overview

The `PortfolioState` acts as the single source of truth for the entire account's value and performance in the APEX V3 architecture. It tracks balance, equity, margin utilization, historical PnL, and exposure limits. It operates similarly to a hedge fund accounting ledger, guaranteeing mathematically rigorous state transitions without silently dropping errors or entering invalid states.

## Structures and Types

### `PortfolioState`
The core structure containing all financial data.
- **Precision:** `rust_decimal::Decimal` is used universally to avoid floating-point errors.
- **Derivatives:** `equity`, `free_margin`, and `margin_level` are continuously recalculated from absolute sources (`balance`, `used_margin`, `floating_pnl`).

### `PortfolioSnapshot`
A versioned, timestamped, immutable representation of the `PortfolioState`. Every valid state transition generates a new snapshot linked to the exact `PortfolioEvent` that caused it. This creates a fully auditable and deterministic history.

### `PortfolioRegistry`
The thread-safe owner of the `PortfolioState`.
- Implemented using `Arc<RwLock<PortfolioState>>` for concurrent read access.
- Uses `DashMap` for concurrent snapshot tracking across different frequency granularities (Realtime, M1, M5, M15, H1, D1).

## Assumptions & Edge Cases

### Margin Level with Zero Used Margin
When `used_margin` is strictly `0`, standard formula `equity / used_margin` results in division by zero. 
**Handling:** We explicitly check for zero and set `margin_level` to `Decimal::MAX` representing infinite or unconstrained margin health.

### Partial Closes
Partial closes emit realized PnL and release a portion of the margin and exposure, acting mathematically identical to a full close but leaving the active positions count unchanged.

### Recovery Transitions
The `RecoveryState` enum (Normal, Recovery, Warning, Critical, Frozen) provides an operational hook for external engines (e.g., Risk Engine) to halt or limit trading activity without polluting the numerical state calculation.

### History Retention
Currently, all `Realtime` snapshots are stored in memory via the `DashMap`. In a production environment, an external async worker or cron job should periodically flush older snapshots to permanent storage (e.g., Redis or PostgreSQL) to prevent Out-Of-Memory (OOM) conditions. The `SnapshotFrequency` enum supports routing snapshots to bucketed aggregation layers.
