# Performance Foundation Tests

## Strategy
This engine forms the mathematical core of the analytics. Testing must guarantee absolute determinism and zero panics.

## Test Suites

### 1. Unit Tests
- Calculate exact metric values against known, manually verified datasets.
- Ensure pure functions behave identically given identical inputs.

### 2. Property Tests
- Utilize `proptest` with `rust_decimal` to blast equations with random `Decimal` values (including limits, MAX, MIN, very small).
- Goal: **Zero Panics**. Any panic is a critical failure.

### 3. Boundary & Safe Math Tests
- Inputs of strictly `0` or `0.0`.
- Missing data paths (calculating Sharpe ratio with 0 trades).
- Division-by-zero paths (Gross Loss = 0).

### 4. Determinism Tests
- Run identical payloads through the event sourcing snapshot reconstructor multiple times.
- Validate the generated hashes/snapshots are byte-for-byte identical every time.
