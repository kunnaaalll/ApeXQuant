# Shadow Mode Invariants

1. **Passive Operation**: Shadow Mode must observe, evaluate, and report. It is strictly forbidden from triggering trades or mutating active state.
2. **Safety**: No `unsafe` code. No `unwrap` or `expect`. No `panic`. All limits bounds are absolute and safely handled.
3. **No Floats**: Core numerical operations must utilize `rust_decimal::Decimal`.
4. **Transition Exclusivity**: `GoLiveValidator` strictly escalates states upon crossing unbroken tracking thresholds. Interrupted streaks regress states dynamically based on current standing.
