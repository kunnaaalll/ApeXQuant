# APEX V3 Capital Allocation Invariants

## Core System Invariants

The Capital Allocation layer is governed by absolute rules. These invariants must hold true at all times during the execution of the portfolio engine.

### 1. Non-Negative Capacity
- `remaining_capacity >= 0`
- `reserved_capacity >= 0`
- `emergency_capacity >= 0`
- `allocation_size >= 0`

Negative capacity or reserves mathematically represent leverage beyond defined constraints and will trigger an `AllocationError::InvariantViolation`.

### 2. State Transition Integrity
- The `AllocationState` transitions must be explicit.
- Example: The system cannot transition directly from `Frozen` to `Aggressive`. It must follow a recovery gradient (`Frozen -> Recovery -> Defensive / Conservative`).
- Violations trigger an `AllocationError::InvalidStateTransition`.

### 3. Reserve Supremacy
- The portfolio can **never** deploy capital if it would cause the total available capital to drop below the sum of the reserves (unless explicitly utilizing the `OpportunityReserve` for an approved setup).
- `CapitalDeployed + Reserves <= TotalCapital`

### 4. Zero Panics & 100% Determinism
- The code must not contain any `unwrap()` on runtime inputs, no `panic!()`, and no `unsafe` code.
- Allocations must yield the exact same decision for the exact same inputs (Heat, Risk Budget, Global Exposure, Reserves) every single time, regardless of platform or time of day.

### 5. Explicit Audit Trail
- Every update to capital allocation state (e.g., changes in reserves, major admitted trades) MUST generate an `AllocationEvent` containing an `AllocationSnapshot`. 
- No hidden capital reservations are allowed.
