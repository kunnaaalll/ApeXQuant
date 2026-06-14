# Exposure Engine Testing Strategy

## Overview
The Exposure Engine enforces rigorous visibility into the distribution of risk across the APEX V3 architecture. Given the complexities of synthetic assets, floating limits, and high-frequency transactions, the test suite guarantees mathematical stability via unit testing, parameter fuzzing, and simulated high-concurrency event replays.

## Test Boundaries

### 1. Currency Decomposition & Synthetic Testing
Located in `tests/exposure_tests.rs`.
- **Purpose:** Validate that a singular synthetic position accurately distributes its risk into constituent base and quote currencies.
- **Methodology:** We mock a `PositionOpened` event for a Long `EURUSD` trade. We assert that:
    - The `EUR` exposure bucket captures `+X` Net/Long/Gross exposure.
    - The `USD` exposure bucket captures `-Y` Net, `Y` Short, and `Y` Gross exposure.
    - The `Global` exposure accurately sums the absolute Gross limits while deriving the proper Net offset.

### 2. Duplicate Detection Validation
Located in `tests/exposure_tests.rs`.
- **Purpose:** Ensure the Concentration logic (`assess_concentration()`) correctly catches correlated or overlapping systemic risks.
- **Methodology:**
    - We feed sequential trades that all share a `USD` quote (e.g., `EURUSD` Long, `GBPUSD` Long). The engine correctly identifies an *Excessive USD short exposure*.
    - We construct a synthetic "Risk-On" portfolio comprising heavy weightings in `BTCUSD` and `NAS100`, successfully triggering the *High Risk-on concentration* assessment.

### 3. Fuzzing Invariants (`proptest`)
Located in `tests/exposure_tests.rs`.
- **Purpose:** Break the invariant boundaries using random floating bounds.
- **Methodology:** A `proptest` generator feeds wild `long` and `short` sizes into the event processor, asserting that the invariant loop `validate_invariants()` perfectly parses boundaries (e.g., validating that total weights sum predictably and Gross mathematically overrides Net constraints).
