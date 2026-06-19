# Risk Health Endpoints

The Health API layer multiplexes onto the same `0.0.0.0:50053` port using `axum`.

## GET `/health`
- **Purpose**: Liveness probe. Indicates if the binary is up and running.
- **Returns**: `{"status": "alive"}` (HTTP 200 OK)

## GET `/ready`
- **Purpose**: Readiness probe. Indicates if the service has connected to PostgreSQL, Redis, and its internal engines are fully loaded in memory.
- **Returns**: `{"status": "ready"}` (HTTP 200 OK) or HTTP 503 if unavailable.
