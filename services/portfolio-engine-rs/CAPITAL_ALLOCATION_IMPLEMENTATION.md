# APEX V3 Capital Allocation Implementation

## Overview

The **Capital Allocation** layer serves as the bridge between Portfolio State, Exposure Engine, Heat Engine, and the ultimate decision to deploy capital. It embodies the mindset of a hedge fund CIO: the primary objective is **capital preservation and maximizing longevity**, not maximizing returns at any cost.

## Allocation Philosophy

1. **Maximize Longevity:** We survive drawdowns by maintaining strict capital reserves. We never fully deploy capital.
2. **Preserve Optionality:** By avoiding overcommitment, the portfolio is always ready for exceptional opportunities or capable of absorbing shock regimes.
3. **Strict Transition Logic:** The portfolio shifts between distinct states (`Aggressive`, `Normal`, `Defensive`, `Conservative`, `Recovery`, `Frozen`). Transitions are explicit; a frozen portfolio cannot suddenly become aggressive without passing through recovery.

## Component Breakdown

### CapitalAllocator
Evaluates incoming requests to deploy capital against a rigorous set of constraints. It checks:
- **Heat Score:** Critical/Frozen heat will immediately block or freeze admissions. Hot heat results in reduced admission sizes.
- **Risk Budget:** Ensure the remaining risk capacity is not breached.
- **Reserve Limits:** Capital cannot be deployed if it would drain the portfolio below minimum reserve levels, unless explicit opportunity reserves are utilized.

Output is a `CapitalAllocationDecision` containing an explicit `can_accept_trade` boolean and an `admission_decision` explaining the exact reason for the outcome.

### ReserveManager
Maintains distinct buckets of reserves:
- **Normal Reserve:** Baseline dry powder.
- **Emergency Reserve:** Absolute minimum capital required for catastrophic survival.
- **Recovery Reserve:** Additional buffer required while recovering from a drawdown.
- **Opportunity Reserve:** Capital explicitly unlocked only for high-conviction or exceptional setups.

### AllocationRecoveryModel
A deterministic model for managing drawdowns.
- If the drawdown exceeds the threshold, the portfolio enters a state of restricted allocation.
- The recovery decays slowly. Even if PnL slightly improves, the restriction persists and decays sequentially via `tick_decay`. This ensures we do not prematurely accelerate capital deployment during choppy recovery markets.

## Event Sourcing
Every allocation change generates an `AllocationEvent` and an immutable, versioned `AllocationSnapshot`. There are absolutely no silent mutations to capital state. Every capital commitment is tracked, auditable, and replayable.
