# Streak Engine Implementation

The Streak Engine (`src/streaks/mod.rs`) is responsible for keeping track of both winning and losing streaks to evaluate performance momentum and apply recovery constraints.

## Components

### StreakDetector
Tracks:
- `current_win_streak`
- `current_loss_streak`
- `average_win_streak`
- `average_loss_streak`

Calculates a deterministic `StreakImpact` (`Positive`, `Neutral`, `Negative`, `Critical`) depending on the severity and length of ongoing streaks.

### RecoveryFactor
Ensures that recovery from significant drawdowns or negative confidence penalties remains gradual. Recovery is inherently limited to guarantee that single wins do not fully recover confidence after significant institutional drawdowns.

## Guarantees
- 100% deterministic (no randomness, no floating point arithmetic).
- Utilizes `rust_decimal::Decimal` exclusively.
- Zero-panic guarantees.
