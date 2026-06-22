# Execution Anomaly Detection

A deterministic, rule-based engine designed to flag severe execution drift.

## Detectable Anomalies
- Spread explosion
- Slippage spikes
- Fill deterioration
- Liquidity collapse
- Latency spikes
- Rejection bursts
- Repeated timeouts

## Output
Emits an `ExecutionAnomalyReport` containing a fixed `Severity` (Minor, Warning, Major, Critical, Catastrophic). Uses zero ML and zero randomness.
