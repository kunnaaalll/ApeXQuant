# APEX V3 Capital Allocation Tests

## Testing Strategy

The Capital Allocation layer defines the survival of the portfolio, hence testing must be exhaustive, deterministic, and free of race conditions or panics.

### Unit Tests
- **Reserve Manager Initialization:** Verifies that negative reserves throw invariant errors immediately.
- **Recovery Decay:** Validates the slow recovery progression when drawdown thresholds are breached and recovered.
- **Trade Admission Heat Validation:** Ensures that `Critical` heat rejects trades, and `Hot` heat scales down requested capital.
- **Trade Admission Risk Validation:** Ensures trades are rejected if risk budgets are exceeded.
- **Opportunity Reserve Utilization:** Tests that high-conviction trades can successfully tap into opportunity reserves when normal reserves restrict allocation.

### Property & Fuzz Tests (To Be Implemented)
- **Fuzzing Allocation Size:** Feeding massive allocations, negative requests, and zero values to ensure the `CapitalAllocator` safely rejects them without panicking.
- **Deterministic State Machine:** Ensuring `AllocationState` transitions mathematically map input heat/drawdown states without variance.

### Stress Tests (To Be Implemented)
- **Mass Admission Scenario:** Admitting thousands of trades under constraints to ensure average latency `<1 ms` and P99 `<5 ms`.
- **Low Margin Regimes:** Simulating extreme margin pressure against reserves to validate rejection logic under duress.

### Replay Tests (To Be Implemented)
- Replaying historical `AllocationSnapshot` and `AllocationEvent` structures to guarantee backward compatibility and deterministic state restoration.
