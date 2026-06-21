# Smart Routing Implementation

Determines the deterministic flow of an order from creation to a venue.

1. Takes `Urgency` and `Priority`.
2. Computes the optimal `RoutingState` (Primary, Secondary, Fallback).
3. Evaluates venue configurations.
4. Returns a `RoutingDecision` containing the target venue and state.
