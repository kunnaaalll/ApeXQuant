# Circuit Breaker Invariants

The `risk-engine-rs` Circuit Breaker strictly adheres to institutional security constraints. The defense system must never become a source of risk itself.

## 1. Zero Panics
There are no `unwrap()` or `expect()` calls allowed anywhere within the `circuit_breaker` module (excluding deterministic tests). State transitions that fail emit an `Err(CircuitBreakerTransitionError)` rather than crashing the application.

## 2. Zero Unsafe Code
Enforced by `#![deny(unsafe_code)]` at the library root. Memory safety is delegated entirely to the Rust compiler without bypasses.

## 3. Decimal Arithmetic Exclusively
No `f32` or `f64` types. All financial math utilizes `rust_decimal::Decimal`. This eliminates IEEE-754 floating-point inaccuracies, non-associativity, and rounding errors that compound over thousands of trades.

## 4. Bounded Risk
Risk restrictions and leverage caps are strictly bounded. Negative leverage or negative drawdown capacity is an invariant violation. All module-level logic explicitly handles sub-zero overflows by clamping to zero.

## 5. Event Driven Snapshotting
No mutation occurs without a corresponding `CircuitBreakerEvent` representation. The entire engine state can be losslessly converted to a Snapshot, ensuring regulatory requirements for "explainability" and "auditability" are mathematically provable.
