# REGIME ANALYSIS IMPLEMENTATION

## Mathematical Foundations
The Regime module evaluates trading performance across strictly defined market states: Trending, Ranging, Expansion, Contraction, High Volatility, and Low Volatility.

### State Evaluation
State determinations (Exceptional, Strong, Normal, Weak, Negative) are calculated via absolute mathematical bounds:
- **Exceptional**: Expectancy > 0 and Profit Factor >= 2.0
- **Strong**: Expectancy > 0 and Profit Factor >= 1.5
- **Normal**: Expectancy > 0 and Profit Factor > 1.0 (or trade count < 30)
- **Weak**: Expectancy <= 0 and Profit Factor > 0.0
- **Negative**: Profit Factor <= 0.0

### Safe Math Assumptions
- `rust_decimal` is used to prevent float imprecision.
- Division operations are checked for zero denominator outside this specific module to prevent panics.
- Outputs are bounded; stability index and drawdown percentages are guaranteed finite.

### Known Limitations
- The engine relies on external tagging of what "Regime" a trade occurred in.
- Small sample sizes (n < 30) bypass strict grading and default to `Normal` to prevent misclassification.
