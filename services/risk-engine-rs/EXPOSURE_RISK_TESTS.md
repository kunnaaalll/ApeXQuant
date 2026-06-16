# Exposure Risk Engine Testing

Testing for the Exposure Risk Engine enforces deterministic behavior, numerical correctness, and safety invariants.

## Verified Invariants
1. **Symbol exposure calculations**: Net and gross exposures are correctly aggregated.
2. **Synthetic currency decomposition**: Forex pairs correctly map to long base, short quote structures.
3. **Sector concentration**: Accurately computes sector dominance percentage.
4. **Theme clustering**: Identifies non-obvious cross-asset thematic clusters.
5. **Gross >= Net invariant**: `Gross Exposure >= abs(Net Exposure)` is strictly verified.
6. **Score clamping**: Concentration and diversification scores never exceed the `[0, 100]` domain.
7. **Determinism over 100,000 evaluations**: Verified that re-evaluating the exact same metrics repeatedly always yields the exact same state and score without variance or drift.
8. **Snapshot replay correctness**: Snapshots properly capture state and events for perfect replayability.
9. **Zero panics**: All transitions are safe, and frozen states correctly latch without throwing runtime errors.
10. **Short USD Clusters**: Appropriately flags large aggregate negative net exposures against the US Dollar.

Tests are located in `src/exposure/tests.rs`.
