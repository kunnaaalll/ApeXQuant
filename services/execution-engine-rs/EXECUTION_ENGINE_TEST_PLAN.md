# EXECUTION ENGINE TEST PLAN

## 1. Unit Tests
- Every state transition must be covered by a unit test.
- Every order type (Market, Limit, Stop, etc.) must have placement, modification, and cancellation tests.
- Trade management components (Stop Loss, Take Profit, Breakeven, Trailing) must have discrete logic tests.

## 2. Property Tests
- Test edge cases in math (partial fill sizes, fractional targets).
- Test rapid state transitions ensuring determinism.

## 3. Replay Tests
- Capture historical order flows and replay them through the state machine.
- Verify identical outputs for identical inputs.

## 4. Network Failure & Broker Disconnect Tests
- Simulate MT5 bridge drops, timeouts, and HTTP 500s.
- Validate the Retry module and Idempotency keys (no duplicate orders).
- Test recovery and state synchronization upon reconnection.

## 5. Reconciliation Tests
- Inject desyncs (missing fills, phantom positions) and verify `reconciler.rs` correctly identifies and repairs them.

## 6. Stress & Determinism Tests
- Send 10,000+ rapid events.
- Ensure average latency remains < 2ms and P99 < 10ms.
- Ensure memory stability over long-running simulated tests.
