# Strategy API Invariants

The API transport layer in Phase 8 strictly conforms to deterministic and stateless constraints to maintain `strategy-engine-rs` as a purely computational module.

## 1. Zero Business Logic in Transport
- **Enforcement**: The API layer `service.rs` merely unpacks Protobuf message structs and delegates to the underlying `strategy` core components without any modification, condition checking, or complex logic.
- **Why**: Keeps the strategy logic testable completely isolated from network transport.

## 2. No Floats (Deterministic Financial Math)
- **Enforcement**: Both gRPC (`strategy.proto`) and HTTP types strictly avoid `f32`/`f64`. Instead, we represent fractional configurations as `string` values inside the protobuf definitions.
- **Why**: Eliminates nondeterminism and floating-point errors. `rust_decimal::Decimal` is exclusively used in the application layer.

## 3. Graceful Error Handling
- **Enforcement**: Zero `panic!()`, `unwrap()`, or `expect()` anywhere in the API layer. All potential failures map directly to standard `tonic::Status` codes.
- **Why**: Ensures maximum uptime.

## 4. `#[deny(unsafe_code)]`
- **Enforcement**: Configured globally in the crate and strictly adhered to in the network parsing boundaries.

## 5. Strict Separation of Concerns
- **Routing**: `server.rs` exclusively handles `axum` and `tonic` routing.
- **Logic**: `service.rs` implements `StrategyService` without domain calculations.
- **Interception**: Logging, metrics, and auth run independently in `interceptors/`.
