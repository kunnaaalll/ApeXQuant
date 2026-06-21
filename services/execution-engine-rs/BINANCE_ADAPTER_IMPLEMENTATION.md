# Binance Adapter Implementation

The Binance Adapter routes requests to Binance Spot and Futures via standard `BrokerAdapter` methods.

- **Path**: `src/brokers/binance`
- **Fields**: Tracks Binance specific states such as `wallet_balance`, `unrealized_pnl`, `maintenance_margin`, `initial_margin`.
- **Margin Logic**: Implements specific transformations to convert Binance metrics into generalized `margin_level`.
- **Events**: `BinanceEvent` wraps standard `BrokerEvent`.
