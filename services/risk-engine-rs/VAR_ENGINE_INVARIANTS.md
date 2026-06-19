# VaR Engine Invariants

The `var` subsystem strictly adheres to the following safety and mathematical invariants:

## Mathematical Invariants
- `VaR >= 0`: VaR represents a loss magnitude; thus, it must be non-negative.
- `ExpectedShortfall >= VaR`: The conditional expected loss beyond VaR must logically exceed or equal the VaR threshold.
- `TailRiskScore ∈ [0, 100]`: Scoring logic is strictly clamped.
- `LargestLoss >= AverageTailLoss`: An axiomatic constraint of tail distributions.

## Safety Invariants
- **No Floating Point Arithmetic**: Everything utilizes `rust_decimal::Decimal`.
- **No `NaN` or `Infinity`**: Handled natively by Decimal and guarded against.
- **No `panic!()`**: Checked division and robust fallback to `Decimal::ZERO`.
- **Zero Unsafe Code**: `#![deny(unsafe_code)]` at the crate root.
- **Strict Clippy Constraints**: `cargo clippy` must pass with zero warnings (`-D warnings`).
