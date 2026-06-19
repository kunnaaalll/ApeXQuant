# Strategy Interceptors Implementation

The `interceptors/` module implements middleware and request-level interception for `strategy-engine-rs`.

## 1. Authentication Interceptor (`auth.rs`)
Implemented directly using Tonic's lightweight `tonic::service::Interceptor`.
- **Purpose**: Validates incoming gRPC requests for valid API keys.
- **Constraints**: Contains no business logic.
- **Behavior**: Retrieves `authorization` from metadata. Rejects requests with `Status::unauthenticated` if empty or invalid.

## 2. Metrics Middleware (`metrics.rs`)
Implemented using `tower::Layer` and `tower::Service` over `tonic::codegen::http::Request`.
- **Purpose**: Generates and captures metrics for requests.
- **Data Captured**: Start time, duration (latency), request success tracking.
- **Compliance**: We bypass mismatched `http` version bound failures between `axum` and `tonic` by specifically deriving against `tonic::codegen::http`.

## 3. Logging Middleware (`logging.rs`)
Implemented identically to `metrics.rs` using `tower::Layer`.
- **Purpose**: Standard request/response logging for structured observability.
- **Data Captured**: Endpoints, timing, and system state boundaries.
- **Constraints**: No complex introspection of payloads (which ensures deterministic behavior without payload decoding overhead).

## Testing
All interceptors are validated in `tests/api_tests.rs` with mock requests ensuring that valid paths succeed, unauthorized paths fail, and middleware state mutations (future captures) don't violate thread-safety (`Send`).
