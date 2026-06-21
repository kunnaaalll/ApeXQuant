# Broker Health Model

Health status dictates routing decisions. The `BrokerHealth` struct relies entirely on `rust_decimal::Decimal` to avoid rounding errors and floating point non-determinism.

## Metrics
- `latency_ms`: Response time threshold.
- `uptime_percentage`: Uptime derived via Decimal metrics.
- `heartbeat_interval_ms`: Scheduled heartbeat delay.
- `last_response_time`: `SystemTime` tracking.
- `reconnect_attempts`: Counter for automatic failovers.

## Thresholds
- **Healthy**: `uptime_percentage >= 95.0%` AND `latency_ms < 500ms`.
- **Degraded**: `uptime_percentage < 95.0%` OR `latency_ms >= 500ms`.
