# Execution Shadow Health Model

Aggregates overall component parity quality into distinct tiers:

- **Excellent:** Parity Score `≥ 99.0`
- **Good:** Parity Score `≥ 95.0`
- **Normal:** Parity Score `≥ 90.0`
- **Weak:** Parity Score `≥ 80.0`
- **Critical:** Parity Score `< 80.0`

Note: A system might score `Excellent` based purely on statistics, but if the `GoLiveValidator` reveals zero recent parity streaks (recent extreme failures), the Health mapping engine enforces a hardcap limiting the maximum assigned health to `Good` to prompt investigation.
