# MARKET STATE MACHINE

## Overview
Evaluates macro conditions and enforces valid state transitions to prevent illogical jumps in the state machine.

## States
1. `Healthy`
2. `Normal`
3. `Stressed`
4. `Dislocated`
5. `Closed`

## Transition Invariants
- Cannot jump directly from `Healthy` to `Dislocated`.
- Cannot jump from `Closed` directly to `Stressed` or `Dislocated`.
- Transitions utilize `Result<MarketState, &str>` and fail explicitly if invalid.
