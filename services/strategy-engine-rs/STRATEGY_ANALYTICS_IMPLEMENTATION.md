# Strategy Analytics Implementation

The `StrategyAnalytics` module within `src/analytics/mod.rs` produces high-level aggregations and derived metrics representing the broader momentum of a strategy.

## Components
Calculates and preserves strictly via `rust_decimal::Decimal`:
- `confidence_acceleration`
- `degradation_acceleration`
- `recovery_speed`

Also identifies structurally categorical properties:
- `strongest_dimension`
- `weakest_dimension`

## Guarantees
- Completely deterministic derivations.
- Utilizes `rust_decimal::Decimal` exclusively.
- Zero-panic guarantees.
