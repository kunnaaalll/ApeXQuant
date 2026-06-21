# Execution Broker Layer Implementation

The `execution-engine-rs` now incorporates a deterministic Broker Connectivity Layer under `src/brokers`.
This layer ensures that order execution can be routed to either `MT5` or `Binance` via a unified `BrokerAdapter` interface.

## Principles
1. **Determinism**: No floats are allowed, all states representable and exact using `rust_decimal::Decimal`.
2. **Stateless Logic**: The adapters do not contain trading logic. The Execution Engine holds the source of truth, passing requests through adapters for routing.
3. **No Hidden Failures**: Errors are explicit. `unsafe`, `panic!`, and `unwrap()` are banned. `Result` is used for all state changes.

## Module Structure
- **registry**: Contains `ExecutionRouter`, `BrokerRegistry`, and `FailoverState` logic for managing multiple broker instances.
- **mt5**: The adapter mapping `BrokerAdapter` to MT5 concepts.
- **binance**: The adapter mapping `BrokerAdapter` to Binance (Spot/Futures) concepts.
- **Core Components**: `connection.rs`, `health.rs`, `events.rs`, `snapshots.rs`, `errors.rs`, `responses.rs`, `requests.rs`.
