# Execution Engine Tests

We cover all invariants and business logic using exact tests.

- **Determinism**: 100,000 iterations computing truncated sums and geometric growths to ensure zero drift across executions.
- **Slippage**: Exact boundary checks (0, 50, 100).
- **Splitting**: Verification that splits preserve exact order quantity without lost decimals.
- **Transitions**: Validation that IOC/FOK/etc reject illegal state transitions.
- **Snapshots**: Exact event-sourced replay reconstructions.
