# Risk Go-Live Certification

Before promoting the `risk-engine-rs` to production, this go-live certification verifies the completeness of validation and stress tests.

## Steps to Go-Live
1. **Run Validation Test Suite**: All unit and integration tests under `tests/validation_tests.rs` must pass without failures.
2. **Compile-time Checks**: The CI pipeline must strictly pass `cargo clippy --all-targets --all-features --no-deps` with 0 warnings.
3. **Parity Sign-off**: Legacy team must confirm the `Reporter` output demonstrating > 99% logic parity.
4. **Stress Sign-off**: Run the Monte Carlo permutation generator for 48 hours to guarantee absolute absence of state corruption or panicking logic.
5. **Final Reporter Output**: Generate the `ValidationReport` json/markdown detailing the final verified measurements.
