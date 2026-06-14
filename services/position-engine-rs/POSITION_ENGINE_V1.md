# APEX V3 — POSITION ENGINE V1

Signal Engine V1 certified.
Risk Engine V1 certified.
Execution Engine V1 certified.
Position Engine is now authorized.

## Role & Responsibilities

The Position Engine owns trade lifecycle management. While the Execution Engine manages individual orders, the Position Engine manages holistic positions. It thinks like a professional trader managing active positions, not a broker managing orders.

### Engine Ownership
- **Position Tracking**: Entry price, current price, position size, PnL (realized and unrealized), risk/reward metrics.
- **Trade Lifecycle**: Explicit state transitions covering Opening, Active, Scaling, Reducing, Closing, etc.
- **Health & Quality**: Calculating Health Scores and categorizing trade quality from Excellent to Critical.
- **Scaling Strategies**: Assessing whether scaling in or out is justified based on trade improvement and risk levels.
- **Exit Recommendations**: Generating recommendations for reducing or fully closing the position.
- **Trade Aging**: Detecting stale, overextended, or underperforming trades based on holding efficiency and expected duration.

### Engine Exclusions
Position Engine does **NOT** own:
- Signal generation
- Risk calculations
- Order routing
- Portfolio management
- AI decisions

## Architecture Overview

The system is designed with a deterministic state machine at its core to ensure zero hidden transitions.

### Module Structure
- `positions/`: Core data structures (`tracker`, `state`, `registry`).
- `lifecycle/`: State machine handling (`transitions`, `events`).
- `health/`: Trade diagnostics (`health_score`, `quality`, `momentum`, `aging`).
- `management/`: Active management logic (`scale_in`, `scale_out`, `reduce`, `close`).
- `pnl/`: Financial metrics tracking (`unrealized`, `realized`, `metrics`).
- `analytics/`: Historical and predictive statistics (`position_stats`, `trade_stats`, `holding_period`).
- `storage/` & `api/`: Postgres persistence and gRPC external interfaces.

## Go-Live Requirements

The Rust Position Engine runs in parallel with the TypeScript implementation in Shadow Mode.

- Health agreement: >95%
- Quality agreement: >95%
- Recommendation agreement: >95%
- Deterministic outputs: 100%
- Stable memory footprint
- Zero panics
- Zero unsafe code block usage

Only after these criteria are successfully met and documented will Portfolio Engine V1 begin development.
