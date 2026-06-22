# Trade Guards

The `trade_guards` module functions as a pure recommendation layer. It aggregates decisions from spread, liquidity, latency, slippage, fill quality, and failure tracking.

## Guard Actions
Based on the checks, the pre-trade evaluation yields one of the following deterministic recommendations:
- `Allow`
- `ReduceSize`
- `Delay`
- `SplitOrder`
- `Block`
- `FreezeTrading`

This module never initiates trades itself; it exclusively blocks or alters execution context.
