# Execution Go-Live Criteria

Before activating institutional volume flows:

1. **Deterministic Exactitude:** `100,000` consecutive simulated orders must yield exactly zero state deviations.
2. **Acceptable Drift Bounds:** Relative Execution drift must maintain an overall rating of `None` to `Low` across a 30-day lookback.
3. **Panic-free Robustness:** Floating-point precision traps must be neutralized (`rust_decimal` used exclusively).
4. **Go-Live Validation States:**
   `NotReady -> Monitoring -> Candidate -> Approved`.
   Approval requires an unbroken 100-streak parity match run. Any failure recursively drops status backward.
