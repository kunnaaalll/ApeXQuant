# Execution Shadow Implementation

## Overview
The Shadow Mode operates by replicating logic with zero side-effects. It tracks metrics and drift, comparing results deterministically.

## Architecture
- `comparison.rs`: Pure observation of state differences.
- `drift.rs`: Decimal-based tracking bounded [0, 100].
- `statistics.rs`: Match rates tracked strictly.
- `validator.rs`: State machine for monitoring go-live readiness.
