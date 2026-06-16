# Performance Shadow Mode Implementation

## Architecture
The Shadow Mode system isolates the Rust performance engine into an observation-only instance running concurrently with the TypeScript legacy engine.

### Data Flow
1. Trade/Tick events are dispatched to the TypeScript engine.
2. The same events are replicated to the Rust `shadow_storage.rs`.
3. The TypeScript engine finishes and outputs Metrics (Expectancy, Confidence, Edge).
4. The Rust engine evaluates its state.
5. The `ShadowValidator` compares both outputs using a strict tolerance defined via `rust_decimal::Decimal`.

### Drift Analysis
- **`drift.rs`**: Computes absolute and relative drift between the legacy value and the rust value. 
- **`comparison.rs`**: Classifies drift into `ExactMatch`, `CloseMatch`, `Warning`, `Mismatch`, or `Critical`.
- **`statistics.rs`**: Cumulatively stores total mismatches, peak drift, and overall agreement percentage.
- **`reporter.rs`**: Serializes statistics to disk as JSON/MD.

### Certification Tie-in
The parity results from the shadow mode are directly fed into `PerformanceParityValidator`, which informs the Go-Live Certification engine.
