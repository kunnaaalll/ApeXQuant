# Latency Protection

Monitors connection health dynamically to protect against network drops or broker issues.

## Metrics
- Broker latency (ms)
- Exchange latency (ms)
- Network latency (ms)

## Thresholds (Total Latency)
- **Healthy**: < 20ms
- **Warning**: 20-50ms
- **Restricted**: 50-100ms
- **Critical**: 100-200ms
- **Frozen**: > 200ms

Outputs a bounded latency score `0-100`.
