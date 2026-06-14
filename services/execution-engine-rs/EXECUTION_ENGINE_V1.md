# APEX V3 — EXECUTION ENGINE V1

## Overview
The Execution Engine is the most reliability-critical service in the APEX V3 platform. Its purpose is to faithfully execute approved decisions in a deterministic and fault-tolerant manner. It operates similarly to an institutional prime broker execution gateway, not a strategy engine.

## Core Responsibilities
- **Order Placement & Modification**
- **Stop Loss & Take Profit Management**
- **Breakeven Logic & Trailing Stops**
- **Partial Closes**
- **Reconciliation & Recovery**
- **Retry Logic (Idempotent)**
- **Deterministic Execution State**

The Execution Engine **does not** own signal generation, risk calculations, portfolio analytics, learning, or AI components.

## Performance Targets
- **Average latency:** < 2 ms
- **P99 latency:** < 10 ms
- **Panics:** 0
- **Unsafe code:** 0
- **Deterministic behavior:** 100%

## Go-Live Requirements
- Order agreement: > 99% (with Shadow Mode TS engine)
- No duplicate orders
- No phantom positions
- No lost fills
- Deterministic state transitions
- Stable memory usage
- Zero panics

## Module Structure
- `src/orders/`: Handles various order types (Market, Limit, Stop, Modify, Cancel).
- `src/execution/`: The core executor, dispatcher, and routing logic to the MT5 bridge.
- `src/reconciliation/`: Cross-checks internal state against broker state, repairs desyncs.
- `src/trade_management/`: Tactics for managing active trades (SL, TP, breakeven, trailing).
- `src/state_machine/`: Strict, deterministic FSM for order/trade lifecycles.
- `src/retry/`: Idempotency, exponential backoff, and duplicate prevention.
