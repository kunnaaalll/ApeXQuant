# APEX V3 PORTFOLIO ENGINE: SHADOW MODE TESTS

To achieve absolute institutional-grade reliability, the Shadow Mode must pass a comprehensive suite of rigorous testing frameworks.

## 1. Unit Tests
- **Comparison Logic:** Assert that `ExactMatch`, `CloseMatch`, and `Mismatch` correctly classify synthetic differences in State, Heat, and Recommendations.
- **Drift Calculation:** Validate that mathematical aggregations for drift correctly trigger `Watch`, `Elevated`, and `Critical` states.
- **Reporter Output:** Ensure that markdown and JSON reports accurately serialize the expected `ShadowStatistics`.
- **Alert Generation:** Assert that breaches in drift thresholds correctly yield a `ShadowAlert`.

## 2. Integration Tests
- **Storage Appending:** Test that the Postgres/SQLite `ShadowStorage` implementation correctly stores and retrieves sequential events.
- **Storage Constraints:** Validate that mutations or deletions to historical `ShadowSnapshot` records are impossible (Append-only).

## 3. Replay Tests
- Capture a rolling 24-hour sequence of TypeScript production outputs.
- Feed the raw inputs into the Rust engine.
- Diff the Rust engine output against the captured TypeScript output.
- Result must be exactly the recorded `ComparisonResult`.

## 4. Monte Carlo Tests
- Generate 1,000,000 randomized but structurally valid TS output permutations.
- Run them through the `PortfolioComparison` engine.
- Verify that **zero panics** occur during comparison or parsing.

## 5. Stress Tests
- Inject 10x the normal volume of portfolio state updates.
- Monitor `ShadowStorage` write latency to ensure P99 < 20ms.

## 6. Determinism Tests
- Run the same replay sequence 5 times in isolated threads.
- Assert that the cryptographic hash of the resulting `ShadowSnapshot` chain is exactly identical every single time.

## 7. Fuzz Tests
- Send malformed JSON, truncated floats, and structural nonsense to the Comparison Engine.
- Assert that the engine elegantly yields `MissingData` or gracefully logs an error, but never panics.

## 8. Shadow Parity Tests
- Live shadow run over 7 days.
- Compare live aggregated stats against `PORTFOLIO_GO_LIVE_CRITERIA.md`.
