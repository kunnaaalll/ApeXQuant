# Risk Engine API Tests

The API tests are located in `tests/api_tests.rs` and cover the following invariants:

## Error Mapping
Validates that `ApiError` enum variants perfectly map to their corresponding `tonic::Status` error codes (e.g., `ApiError::NotFound` -> `Code::NotFound`).

## Streaming
Validates that the streaming endpoints return a valid `ReceiverStream` without panicking, simulating empty loads.

## Health
Validates that `liveness_check` and `readiness_check` return valid HTTP 200 responses with the correct schema structure.

## Determinism
Simulates 100,000 requests against a gRPC handler to guarantee zero divergence in output structure or error behavior.

## Interceptors
Validates the `AuthInterceptor` against missing, valid, and malformed header metadata.
