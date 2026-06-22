# Execution Shadow Invariants

1. **Zero Side Effects**: Shadow execution must never trigger live trades.
2. **Deterministic Fallback**: Missing data implies conservative estimates.
3. **Immutability**: All tracked events are append-only.
4. **Zero Panics**: The shadow engine never panics or unwraps.
