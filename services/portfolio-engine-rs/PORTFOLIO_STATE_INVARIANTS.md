# Portfolio State Invariants

## Core Principles

The APEX V3 Portfolio Engine enforces rigorous mathematical invariants inside the `PortfolioState` implementation. If any invariant is violated during a state transition, the engine refuses the transition and returns a `PortfolioError`, protecting the core system from propagating invalid states to downstream layers.

## The Invariants

### 1. `Equity = Balance + FloatingPnL`
Equity must exactly equal the realized cash balance plus the active floating profit or loss. Any mismatch triggers `PortfolioError::InvalidEquity`.

### 2. `FreeMargin = Equity - UsedMargin`
Free Margin represents the capital available for new positions. It must perfectly equal the account's total equity minus the margin already allocated to open positions. Any mismatch triggers `PortfolioError::InvalidFreeMargin`.

### 3. `MarginLevel > 0`
The Margin Level represents the health of the account (`Equity / UsedMargin`). It is strictly prohibited from dropping below 0, as negative equity requires immediate liquidation or structural intervention. A violation triggers `PortfolioError::NegativeMarginLevel`. If `UsedMargin` is `0`, the level is effectively infinite and defaulted safely to `Decimal::MAX`.

### 4. `ActivePositions >= 0`
You cannot close more positions than you have opened. The active position counter cannot fall below zero. A violation triggers `PortfolioError::NegativeActivePositions`.

### 5. `PeakEquity >= Equity`
Peak equity tracks the high-water mark of the account. Current equity must mathematically be less than or equal to the peak equity. If current equity surpasses peak equity, peak equity is immediately ratcheted upwards. If an invalid value is manually injected, it triggers `PortfolioError::PeakEquityLowerThanEquity`.

### 6. `Drawdown >= 0`
Drawdown represents the percentage or absolute drop from the high-water mark. It cannot mathematically be negative, as a negative drawdown implies we are above peak equity (which would violate Invariant 5).

## Failure Scenarios

If an external event attempts to inject bad math (e.g., closing a position that wasn't open, emitting a PnL update that artificially manipulates Equity without touching Floating PnL), the `apply_event` method will execute `validate_invariants()` before confirming the state transition. If an error is caught, the state mutates are contained to a local scope (within the event processing), and the error is returned to the caller, preventing the creation of an invalid `PortfolioSnapshot` and keeping the global `PortfolioRegistry` untouched.
