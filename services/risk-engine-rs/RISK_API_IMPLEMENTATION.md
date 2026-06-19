# Risk Engine API Layer Implementation

The Risk Engine API Layer is a stateless transport layer designed to strictly decouple business logic from external integration.

## Architecture

1. **Protobuf Definitions (`risk.proto`)**: Declarative definitions of unary and streaming endpoints.
2. **gRPC Server (`RiskServiceImpl`)**: generated via `tonic`, implements mapping request payloads to internal engine structures, and constructs responses from engine outputs. It performs zero actual risk calculations.
3. **Interceptors**: 
    - `AuthInterceptor`: Validates tokens using standard `tonic::Interceptor`.
    - `LoggingLayer`: Custom `tower::Service` handling request/latency tracing.
    - `MetricsLayer`: Custom `tower::Service` handling Prometheus metrics (`grpc_requests_total`, `grpc_errors_total`, `grpc_request_duration_seconds`).
4. **Health Probes (`/health`, `/ready`)**: Uses `axum` routing for Kubernetes readiness/liveness checks, multiplexed directly onto the same port as the gRPC server.

## Constraints

- Zero business logic inside the API handlers.
- Deterministic behavior: for the same state input, it produces the same output.
- No `unwrap()`, no `panic!`, no `unsafe` code.
