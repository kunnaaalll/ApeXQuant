# Spread Protection

Tracks `current_spread`, `average_spread`, and calculates the `spread_multiplier`.

## State Escalation Based on Multiplier
- **Normal**: <= 1.5x
- **Warning**: 1.5x - 2.0x
- **Restricted**: 2.0x - 3.0x
- **Critical**: 3.0x - 5.0x
- **Frozen**: > 5.0x

Outputs a deterministic `spread_score` bound between `0` and `100`.
