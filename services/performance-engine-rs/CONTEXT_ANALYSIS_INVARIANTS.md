# CONTEXT ANALYSIS INVARIANTS

## Mathematical Invariants
1. **Zero Panics**: Float divisions strictly leverage `rust_decimal` without panicking macros on unbound inputs.
2. **Determinism**: Mathematical derivations must be exact and replayable bit-for-bit.
3. **No Unsafe Code**: Complete absence of `unsafe` blocks.

## Sample Adequacy Invariants
1. `Insufficient` state requires Trade Count < 30.
2. `InstitutionalGrade` state requires Trade Count >= 500.
3. Overfitting penalty approaches 0 as Trade Count -> 500, but evaluates exactly to 1 (maximum penalty) at Trade Count = 0.

## State Invariants
1. No engine state may be `Exceptional` if Expectancy <= 0.
2. Minimum trade constraints inherently bypass `Exceptional` and `Strong` calculations to prevent false confidence.
