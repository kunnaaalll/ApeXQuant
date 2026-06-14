# APEX V3 — Portfolio Quality Implementation

## Overview
The Portfolio Quality engine provides a deterministic, versioned, and event-driven assessment of the portfolio's absolute performance and efficiency. It serves as a report card to measure **how good** the portfolio is performing over time.

## State Model
`PortfolioQualityState` ranges across:
- **Excellent** (>= 90.0)
- **Good** (>= 75.0)
- **Neutral** (>= 50.0)
- **Weak** (>= 25.0)
- **Critical** (< 25.0)

## Quality Breakdown
The composite score (0.0 to 100.0) is constructed from a `PortfolioQualityBreakdown` composed of `QualityContribution`s. Each contribution explicitly defines its `weight`, `score`, and `reason` to prevent black-box calculations.

Factors measured:
- Win rate
- Profit factor
- Expectancy
- Average RR
- Position quality
- Position health
- Capital efficiency
- Trade efficiency
- Holding efficiency
- Allocation efficiency
- Recovery factor
- Recent performance
- Drawdown efficiency

## Decay Model
The `QualityDecayModel` dictates that poor performance or inactivity will gradually reduce the quality score over time. Instant transitions are prohibited; decay is applied linearly and bounded by the `current_score`.

## Snapshots and Events
Every modification creates an immutable `QualitySnapshot` bound to a specific `QualityEvent`.

Events supported:
- `PositionChanged`
- `PnLChanged`
- `HeatChanged`
- `AllocationChanged`
- `RecoveryChanged`
- `CircuitBreaker`
- `VolatilityChanged`
- `DecayApplied`

## Edge Cases and Limitations
- **Oscillation Avoidance**: The decay model combined with smoothed recovery ensures the score does not rapidly ping-pong between states.
- **Dependencies**: This layer depends strictly on state updates. It does not pull data; it expects events to be pushed.
