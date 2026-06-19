# Risk Decision Tree

## Admission Policy Tree
```
Is Drawdown Frozen OR Circuit Breaker Frozen?
 ├── Yes -> FREEZE
 └── No
      ├── Is VaR Critical?
      │    ├── Yes -> BLOCK
      │    └── No
      │         ├── Is Correlation High OR Critical?
      │         │    ├── Yes -> DELAY
      │         │    └── No -> ALLOW
```

## Committee Decision Flow
1. Run Freeze Engine. If triggered, halt all and exit.
2. Run Reduction Engine. If Emergency Reduction is needed, apply severe blocks.
3. Run Block Engine.
4. Run Increase Engine. 
5. Combine outputs to form a consistent, rule-based Recommendation (`RiskCommitteeDecision`).
