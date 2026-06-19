# Confidence Invariants

1. **Clamp Law**: Base ConfidenceScore can never mathematically exceed 100 or drop below 0. Clamping occurs deterministically on instantiation.
2. **Tier Integrity**: Confidence Tier mapping (`VeryLow`, `Low`, `Normal`, `High`, `VeryHigh`) is exhaustive; no score can exist without an exact corresponding tier.
3. **Penalty Monotonicity**: Penalties strictly act as subtractions against Base score. A penalty can never inadvertently increase confidence, and recovery operations strictly pass through `RecoveryFactor` logic.
