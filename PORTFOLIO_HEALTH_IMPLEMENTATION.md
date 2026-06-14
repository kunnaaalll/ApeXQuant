# APEX V3 — Portfolio Health Implementation

## Overview
The Portfolio Health engine evaluates the portfolio's resilience and stability. While Quality answers "how good is the portfolio," Health answers "how safe and resilient is the portfolio." It acts as a defensive barometer.

## State Model
`PortfolioHealthState` ranges across:
- **Excellent** (90-100)
- **Healthy** (75-89)
- **Normal** (50-74)
- **Weak** (25-49)
- **Critical** (0-24)

## Health Breakdown
Health scores are built deterministically from a `PortfolioHealthBreakdown`. Each component provides a `HealthContribution` with a specific `weight`, `contribution`, and human-readable `reason`. 

Factors measured:
- Portfolio heat
- Drawdown
- Margin utilization
- Leverage
- Open risk
- Exposure concentration
- Correlation pressure
- Recovery state
- Circuit breakers
- Capital reserves
- Volatility regime
- Position quality
- Portfolio quality

## Recovery Model
The `HealthRecoveryModel` defines how health recovers after dropping. 
- Recovery must require time and stability.
- No immediate jumps from `Critical` to `Healthy`. 
- Recovery is bounded per tick (e.g., maximum 5 points), ensuring monotonic and gradual improvement.

## Snapshots and Events
All state changes trigger immutable `HealthSnapshot` generation through specific `HealthEvent`s.

Events supported:
- `PositionChanged`
- `PnLChanged`
- `HeatChanged`
- `AllocationChanged`
- `RecoveryChanged`
- `CircuitBreaker`
- `VolatilityChanged`
- `RecoveryTick`

## Edge Cases and Limitations
- **Recovery Bounding**: Maximum tick boundaries prevent spoofed or rapid recovery manipulations.
- **Strictly Unsigned**: The `current_score` operates securely within a `u8` bounded exactly to `100`.
