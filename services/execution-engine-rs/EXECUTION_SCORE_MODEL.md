# Execution Score Model

Scores range from 0 (Worst) to 100 (Best). Combines 4 pillars:
1. **Slippage Score (40%)**: Realized vs Expected Slippage.
2. **Fill Quality (30%)**: Number of slices, partial fill ratios.
3. **Liquidity Quality (20%)**: Depth and spread conditions at execution time.
4. **Latency Score (10%)**: Routing and matching speed.

## Grades
- **Elite**: >= 90
- **Strong**: >= 75
- **Normal**: >= 50
- **Weak**: >= 25
- **Poor**: < 25
