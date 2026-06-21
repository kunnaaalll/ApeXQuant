# Order Domain

The Order domain defines deterministic order structures and state changes.

## Structures
- **OrderId**: Uuid wrapper
- **OrderType**: Market, Limit, Stop, StopLimit
- **OrderSide**: Buy, Sell
- **OrderStatus**: Pending, Submitted, PartiallyFilled, Filled, Cancelled, Rejected, Expired
- **TimeInForce**: GTC, IOC, FOK, Day
