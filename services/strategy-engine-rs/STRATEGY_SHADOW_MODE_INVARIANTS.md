# Strategy Shadow Mode Invariants

1. Total relative difference must always be clamped to `[0, 100]`.
2. Exact Match streaks must zero out immediately upon a warning or mismatch.
3. No panics, unwraps, or floating point calculations are allowed.
