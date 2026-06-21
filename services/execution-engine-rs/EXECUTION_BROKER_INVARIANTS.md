# Broker Layer Invariants

1. **No Panics or Unwraps**: The connectivity layer handles all unexpected network or parsing errors using `Result`. Panicking inside the execution router is unacceptable.
2. **Deterministic State Math**: Use `rust_decimal::Decimal` exclusively. Floating point drift is eliminated.
3. **Strict Transitioning**: Connection and Failover states follow defined DAG sequences. Escaping sequences triggers `BrokerError::InvalidStateTransition`.
4. **Broker Agnosticism**: The engine never directly references MT5 or Binance symbols/orders natively; it communicates through `requests.rs` and `responses.rs`.
5. **Replayable Snapshots**: The broker's state and connection timeline must be fully rebuildable from events.
