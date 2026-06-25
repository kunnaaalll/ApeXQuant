# Momentum Engine

The Momentum engine tracks the velocity and acceleration of price movements to identify exhaustion or impulse phases.

## Mechanics

- Calculates instantaneous returns (`velocity`).
- Stores historical velocities in a ring buffer (`VecDeque`) up to a specified period.
- Compares rolling velocity averages with recent velocities to compute `acceleration`.
- Analyzes divergence between velocity and acceleration to produce a bounded `MomentumGrade` (Extreme, Strong, Moderate, Weak, Neutral).
