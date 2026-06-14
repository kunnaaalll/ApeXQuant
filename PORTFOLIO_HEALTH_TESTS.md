# APEX V3 — Portfolio Health Tests

## Testing Strategy
Because Health dictates capital resilience, the testing methodology requires aggressive simulated adversity.

## Unit Tests
- `test_health_initialization`: Verifies secure default parameters.
- `test_health_state_boundaries`: Evaluates the exact edge mapping for `PortfolioHealthState`.
    - `Excellent`: 90-100
    - `Healthy`: 75-89
    - `Normal`: 50-74
    - `Weak`: 25-49
    - `Critical`: 0-24
- `test_apply_event_clamping`: Verifies that a `u8` bounds securely at 100 maximum, completely avoiding negative underflow risks structurally.
- `test_apply_recovery`: Asserts recovery is gradual. Evaluates the max-recovery-per-tick boundary to prevent jumping from `Critical` to `Healthy` directly.

## Stress and Monte Carlo Tests
- **High Leverage & High Drawdown**: Injects maximum stress events to verify safe degradation into `Critical`.
- **Determinism validation**: Same sequence of events applied multiple times yields exactly the same final `HealthSnapshot`.

## Fuzz Tests
- Applies randomized valid and invalid values for `HealthContribution` objects to ensure the engine gracefully processes bad inputs through clamped behavior rather than panics.
