# Portfolio State Testing Strategy

## Overview

The APEX V3 Portfolio Engine is tested rigorously to ensure total determinism, zero panics, and guaranteed invariant enforcement. The test suite operates across three paradigms: unit tests, property tests (fuzzing), and highly concurrent stress tests.

## Test Categories

### Unit Tests
Located in `tests/state_tests.rs`.
- **Purpose:** Verify basic functional correctness.
- **Coverage:** Tests individual state transitions such as Deposits, Position Openings, and Position Closings. Validates that PnL updates accurately track into equity and margin level calculations.

### Property Tests (Fuzzing)
Located in `tests/state_tests.rs`, powered by `proptest`.
- **Purpose:** Uncover impossible states by fuzzing inputs mathematically.
- **Methodology:** Injects thousands of random valid and invalid numeric values into `balance`, `floating_pnl`, and `used_margin` to verify that `validate_invariants()` correctly traps impossible states without panicking, and correctly permits valid states.

### Determinism & Replay Tests
Located in `tests/registry_tests.rs`.
- **Purpose:** Ensure the event sourcing design can accurately reconstruct the exact portfolio state from a log of events.
- **Methodology:** We feed a sequence of events (deposits, opens, PnL updates, closes) to the `PortfolioRegistry` and assert that the final computed balance, equity, and margin perfectly match manually verified values. We also assert that 1 event = 1 snapshot.

### Concurrency Stress Tests
Located in `tests/registry_tests.rs`.
- **Purpose:** Guarantee thread-safe access under extreme load without global mutable singletons.
- **Methodology:** 10 discrete threads simultaneously spawn and dispatch 100 sets of open, update, and close operations (totaling 4,000 concurrent events) against a single `PortfolioRegistry` `Arc` reference. We verify that the final internal version counter perfectly mirrors the number of events (4001 including the initial deposit), and that the financial state is mathematically flawless.
