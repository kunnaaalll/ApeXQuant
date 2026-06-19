# Strategy Engine API Implementation

The Strategy Engine exposes a robust, deterministic gRPC API for interaction with internal logic. The API layer acts **purely** as a stateless translation boundary.

## Protobuf Definition
Defined in `shared/protobuf/strategy.proto`.
Provides strict deterministic typing (e.g. `apex.common.Decimal` instead of floating point `f32`/`f64`).

## Service Implementation
Located in `src/api/service.rs`.
The `StrategyServiceImpl` handles parsing requests, mapping them to the internal engine inputs, executing them, and mapping the responses back to protobuf formats without executing any business logic.

## Error Handling
Located in `src/api/errors.rs`.
`ApiError` is used internally and maps deterministically to `tonic::Status` with standard gRPC status codes (`NotFound`, `InvalidArgument`, `Internal`, etc.), preventing internal details from leaking.
