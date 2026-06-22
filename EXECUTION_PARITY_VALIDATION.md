# Execution Parity Validation

Parity is computed as a weighted deduction matrix.

- The system starts with a Perfect `100.0` score.
- Exact matches (Diff ≤1%) incur `0` deductions.
- Close matches (Diff ≤3%) incur a `5.0` penalty weight per ratio.
- Warnings (Diff ≤5%) incur a `15.0` penalty.
- Mismatches (Diff ≤10%) incur a `40.0` penalty.
- Critical Mismatches (>10%) wipe score by `100.0`.

```
Level Boundaries:
100.0      => Perfect
95.0 - 99  => Excellent
85.0 - 94  => Good
70.0 - 84  => Acceptable
<70.0      => Poor
```
