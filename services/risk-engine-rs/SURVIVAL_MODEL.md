# SURVIVAL MODEL

The Survival Model serves as the final arbiter combining localized stresses into a solitary health index.

## Score Computation
Starts from an ideal baseline score of `100.0`. Each component engine imposes penalties dynamically based on their respective derived states:

1. **Volatility Penalty:** High, Extreme, and Collapse subtract non-linearly (up to -50.0).
2. **Liquidity Penalty:** Warning, Danger, Critical, and Frozen severely degrade resilience (up to -70.0).
3. **Leverage Penalty:** Danger and Collapse subtract significant chunks for capital instability (up to -60.0).
4. **Correlation Penalty:** When average portfolio correlation exceeds `0.8`, a linear penalty multiplier reduces the score.

The resulting score is strictly clamped between `0.0` and `100.0`.

## Survival States
- `Excellent` (>= 90)
- `Strong` (>= 70)
- `Moderate` (>= 50)
- `Weak` (>= 30)
- `Critical` (>= 10)
- `Failed` (< 10)
