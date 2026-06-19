# STRESS ENGINE INVARIANTS

The following invariants MUST be preserved across all updates and modifications of the stress subsystem:

1. **Deterministic Evaluation:** `fn apply_*` and `fn compute_*` functions must be pure.
2. **Bounds Checking:** Scores and correlation values must naturally map or map through explicit clamps (0-100 for score, -1.0 to 1.0 for correlation).
3. **No Float Imprecision:** `rust_decimal::Decimal` must strictly be utilized in place of `f32`/`f64`.
4. **Immutability:** Emitted `Events` and `Snapshots` are read-only artifacts.
5. **Panic Safety:** Result combinations and option checks must never rely on runtime crashes for flow control.
