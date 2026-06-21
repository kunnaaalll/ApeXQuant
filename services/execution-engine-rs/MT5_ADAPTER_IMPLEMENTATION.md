# MT5 Adapter Implementation

The MT5 Adapter provides a bridge from the Execution Engine to the MT5 servers.

- **Path**: `src/brokers/mt5`
- **Fields**: Tracks MT5 specific states such as `balance`, `equity`, `free_margin`, `leverage`.
- **Conversion**: Converts MT5 objects into generic `AccountInfo`, `SymbolInfo`, `OpenPosition`, `PendingOrder` structures to be consumed by the execution engine.
- **Events**: `Mt5Event` wraps `BrokerEvent` but allows MT5-specific extended events.
