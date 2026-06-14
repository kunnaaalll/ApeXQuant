# APEX V3 Portfolio Heat Engine Invariants

The Portfolio Heat Engine is the bedrock of risk calculation. To ensure absolute safety, the following invariants are strictly mathematically enforced at runtime, with tests ensuring violations can never occur.

## Mathematical Invariants

1. **Heat Score Boundary:**
   `0 <= heat_score <= 100`
   Any sum of `HeatContributionBreakdown` that exceeds 100 is clamped at 100. Heat can never go negative.

2. **Positive Remaining Risk:**
   `remaining_risk >= 0`
   If `utilized_risk + reserved_risk + emergency_reserve > max_portfolio_risk`, then `remaining_risk` is forced to `0`.

3. **Total Risk Capacity Consistency:**
   `max_portfolio_risk == utilized_risk + remaining_risk + reserved_risk + emergency_reserve`
   Every unit of risk must be explicitly accounted for in one of the sub-pools.

4. **Cooldown Enforcement:**
   `if ticks_since_last_loss < cool_down_threshold_ticks => decay_amount == 0`
   Heat cannot organically cool down while the portfolio is still in an active loss cluster.

## System Invariants

1. **Zero Panics:**
   No calculation involving decimal scaling, margin math, or factor division shall invoke a `.unwrap()` or panic sequence on missing data. Graceful fallback logic and safe casting rules apply universally.
   
2. **Immutable Snapshots:**
   Once a `HeatSnapshot` is generated from a `HeatEvent`, its state and values are fundamentally immutable. Any mutation results in a distinct, version-bumped snapshot.
