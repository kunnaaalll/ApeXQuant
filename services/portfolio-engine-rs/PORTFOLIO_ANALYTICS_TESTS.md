# PORTFOLIO ANALYTICS TESTS

This document outlines the testing strategy for the Analytics Engine. 

## Testing Types

1. **Unit Tests**:
   - Every single mathematical operation is individually unit tested against known baseline values.
   - Example: Calculating Sortino ratio with static downside deviation values.

2. **Property Tests**:
   - `proptest` is used to ensure ratios always remain finite and bounding limits are maintained across millions of inputs.
   - Example: Feeding a vast range of win/loss floats to ensure `expectancy` is bounded.

3. **Replay Tests**:
   - Entire sequences of trades are replayed chronologically.
   - Validates that the state of `AnalyticsSnapshot` is perfectly reconstructed.

4. **Monte Carlo Tests**:
   - Randomly generates massive sets of `AnalyticsEvent` to test statistical resilience over simulated periods.

5. **Stress Tests**:
   - Validates latency boundaries. The target is an average latency of < 2ms and a P99 of < 10ms for generating real-time snapshots.

6. **Determinism Tests**:
   - Ensures the exact same series of events processed in a sequence produces the exact identical bitwise representation of `AnalyticsSnapshot`.

7. **Fuzz Tests**:
   - Random unstructured data is fed to the engine to ensure 0 memory leaks, 0 panics, and 0 unsafe execution paths.
