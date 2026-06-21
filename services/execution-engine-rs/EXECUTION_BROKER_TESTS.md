# Execution Broker Tests

The `broker_tests.rs` implements comprehensive deterministic testing for the broker connectivity layer.

## Key Test Cases
- **test_connection_transitions**: Verifies the connection state machine. Only valid transitions are permitted, others return `Result::Err`.
- **test_failover_recovery_sequence**: Validates the failover state sequence (`Healthy` -> `Warning` -> `Failover` -> `Recovery` -> `Healthy`). Ensures direct `Failover` -> `Healthy` transitions fail.
- **test_health_bounds**: Confirms that health checks (latency, uptime) return expected bounds using Decimal representations without floating point math.
- **test_snapshot_rebuild**: Proves serialization/deserialization retains exact state.
- **test_event_determinism**: Confirms identical events hash and equal perfectly.
- **test_mt5_account_state** & **test_binance_account_state**: Confirms accurate conversion and validation of margin balances to a standard `AccountInfo` response.
- **test_determinism_100k_iterations**: A stress test running 100,000 continuous loops to ensure no drift in states.
