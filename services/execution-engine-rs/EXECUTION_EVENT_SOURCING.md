# Execution Event Sourcing

Event Sourcing forms the foundation of APEX V3's replayability. 

Every mutating transition inside the engine generates a corresponding event. These events are wrapped into an `ExecutionEventWrapper` enum and serialized into a PostgreSQL table.

## Payloads
- `OrderEvent`
- `FillEvent`
- `PositionEvent`
- `ExecutionRiskEvent`
- `SmartExecutionEvent`
- `MicrostructureEvent`
- `BrokerEvent`
- `ShadowEvent`

All events derive `Serialize`, `Deserialize`, and `Clone`.
