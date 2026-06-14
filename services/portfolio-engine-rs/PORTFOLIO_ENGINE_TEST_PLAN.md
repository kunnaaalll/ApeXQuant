# PORTFOLIO ENGINE TEST PLAN

- **Unit tests**: Cover pure logic (math, allocation decisions).
- **Property tests**: Fuzz inputs for Heat, Health, and Allocation.
- **Replay tests**: Ensure deterministic transitions from event logs.
- **Portfolio stress tests**: Test behavior under extreme correlations and drawdowns.
- **Determinism tests**: Ensure `f64` math or Decimal math yields identical results across 10,000 iterations.
