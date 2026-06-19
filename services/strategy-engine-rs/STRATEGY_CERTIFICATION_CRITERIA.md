# Strategy Certification Criteria

A strategy becomes a Certified candidate only if ALL five dimensions pass simultaneously:
1. Parity Pass
2. Determinism Pass
3. Replay Pass
4. Stress Pass
5. Benchmark Pass (Average Latency < 2ms, P99 < 10ms)

Any failure immediately resets the strategy state to `NotCertified`.
