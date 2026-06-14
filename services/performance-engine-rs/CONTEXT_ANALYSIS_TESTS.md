# CONTEXT ANALYSIS TESTS

## Test Methodologies

### Unit Testing
Every individual evaluator (`RegimeCalculator`, `SessionCalculator`, `SymbolCalculator`, `TimeframeCalculator`) includes explicit unit tests ensuring state derivation is correct.

### Sample Adequacy Tests
The `SampleAdequacyEngine` tests boundary logic (`MIN_TRADES_INSUFFICIENT`, `MIN_TRADES_RELIABLE`, `MIN_TRADES_INSTITUTIONAL`) to ensure correct state assignment.

### Future Testing
- Property Tests (proptest): Verify associativity and robustness against pseudo-random large/small decimals.
- Determinism Tests: Verify multiple sequential runs emit exact same checksum outputs.
- Boundary Tests: Verify 0 trades correctly triggers safe math returns without panicking.
