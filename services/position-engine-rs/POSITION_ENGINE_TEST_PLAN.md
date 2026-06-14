# POSITION ENGINE V1 - TEST PLAN

## 1. Overview
The testing strategy for Position Engine V1 ensures 100% determinism, zero panics, and mathematical correctness across all position states and transitions.

## 2. Test Categories

### Unit Tests
- **PnL Tests**: Verify mathematically accurate calculations for unrealized/realized PnL, taking fees/slippage into account.
- **State Transition Tests**: Ensure the `PositionState` can only progress through valid pathways (e.g., `Opening` -> `Active` -> `Closed`). Reject invalid paths.
- **Health Tests**: Ensure `HealthScore` outputs valid bounds (0-100) and reacts appropriately to inputs (e.g., drawdown reduces score).

### Property Tests (proptest)
- Fuzz entry price, size, and current price combinations to ensure no math overflows or panics.
- Validate that `unrealized_pnl` always correlates correctly with distance to `current_price`.

### Replay Tests
- Ingest a sequence of historical market events to simulate the full lifecycle of a position and verify the sequence of recommendations (Scale, Reduce, Close).

### Stress Tests
- Concurrently open, update, and close 10,000+ positions simulating high-frequency updates, ensuring latency remains under target parameters (< 3ms avg).

### Determinism Tests
- Running the same sequence of price updates and signal events on two identical engine instances must yield identical state transitions and internal UUID representations.
