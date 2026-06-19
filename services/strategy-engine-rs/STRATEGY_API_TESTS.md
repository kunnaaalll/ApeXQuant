# Strategy Engine API Tests

The API tests are strictly separated from domain tests and verify the stateless transport mechanisms.

## Test Areas

### 1. Endpoint Verification
- Validates that `EvaluateStrategy` roundtrips deterministically.
- Validates exact matching of mapped internal strings and logic.

### 2. Error Mappings
- Tests parsing invalid UUIDs and expects `InvalidArgument` gRPC codes.
- Asserts that internal errors do not leak internal system strings.

### 3. Interceptors
- Validates the `auth_interceptor` blocks unauthorized connections with `Unauthenticated`.
- Implicitly verifies logging and metrics wrappers run without error.

### 4. Deterministic Invariant Checks
- Property check simulating 1000 identical `EvaluateStrategy` calls ensures the response is totally identical and deterministic.

### 5. Health Multiplexing
- Probes `/health` and `/ready` over HTTP confirming concurrent routing alongside gRPC on the same runtime configuration.
