# Risk Certification Criteria

To achieve "Certified" status, the Risk Engine must meet the following criteria:

- **Agreement**: > 99% agreement with the legacy risk engine on all calculated outputs.
- **Panics**: 0 detected panics during testing and load validation.
- **Corruption**: 0 instances of memory or state corruption across all stress tests.
- **Determinism**: 100% (exact identical state output for 100k repetitions).
- **Replay Consistency**: 100% exact match between event sourcing replays and snapshot data.
- **Latency Benchmarks**: 
  - Average latency < 2ms
  - p99 latency < 10ms
