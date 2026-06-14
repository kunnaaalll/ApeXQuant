# Exposure Engine Invariants

## Zero-Tolerance Axioms

The Exposure Engine enforces the following strict mathematical invariants during the `validate_invariants()` call that concludes every state transition:

### 1. `Gross >= Net`
Gross exposure mathematically cannot be less than Net exposure.
- **Why:** Net exposure allows for hedging offsets (e.g., Long 1M EURUSD, Short 1M EURUSD = Net 0). Gross exposure enforces the absolute deployed capital size (Gross = 2M). If Net > Gross, the accounting is corrupted.
- **Enforcement:** `if self.global.gross_exposure < self.global.net_exposure { Err }`

### 2. `Gross >= 0`
Gross exposure can never be negative.
- **Why:** Gross exposure is defined as the absolute sum of all long and short positions. The absolute sum of any real numbers is `>= 0`.
- **Enforcement:** `if self.global.gross_exposure.is_sign_negative() { Err }`

### 3. Maximum Weight Cap (`<= 100%`)
The sum of relative exposure weights for components (e.g., individual symbols) cannot exceed `1.0` (100%) against the defined denominator (global gross exposure).
- **Why:** If the sum of individual weights exceeds 100%, the portfolio is structurally unaccounted for (hidden leverage or broken math).
- **Enforcement:** `let total_weight: Decimal = self.symbols.values().map(|s| s.weight).sum(); if total_weight > Decimal::ONE { Err }`

### 4. Zero Position Exhaustion Floor
The global position count cannot fall below zero.
- **Why:** Releasing more positions than currently tracked indicates phantom closures.
- **Enforcement:** Enforced proactively on `PositionClosed` and `ScaleOut` events before they are applied. `if self.global.position_count == 0 { Err }`

## Certification Protocol
If an invariant failure is triggered, the `ExposureRegistry` will mathematically reject the proposed `ExposureEvent`, abandon the state mutation, and log a critical failure. The state will roll back to the previously snapshotted `ExposureState` implicitly because the `RwLock` is dropped without modifying the version sequence.
