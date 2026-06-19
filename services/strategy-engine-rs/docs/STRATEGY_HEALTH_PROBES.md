# Strategy Health Probes

## Multiplexing Architecture
The strategy engine runs multiple network protocols:
1. **gRPC** (`tonic`): Handles core inter-service communication over HTTP/2.
2. **HTTP** (`axum`): Handles operational and orchestration tasks, including Kubernetes health probes over HTTP/1.1 or HTTP/2.

## Probes Implemented

### 1. `/health` (Liveness Probe)
- **Role**: Validates that the underlying container and process are running and capable of responding to network requests.
- **Implementation**: Defined in `src/api/health/health.rs`.
- **Response**: `200 OK` with JSON payload `{"status": "ok"}`.

### 2. `/ready` (Readiness Probe)
- **Role**: Validates that the service is functionally capable of handling traffic (e.g., dependencies are connected, models are loaded).
- **Implementation**: Defined in `src/api/health/readiness.rs`.
- **Response**: `200 OK` with JSON payload `{"status": "ready"}`.

## Concurrent Execution
Because HTTP/1.1 (Axum) and gRPC (Tonic HTTP/2) multiplexing is complex on a single port due to TLS constraints, we implemented a concurrent dual-server approach. 
In `src/api/server.rs`, we spawn two separate `tokio::spawn` tasks binding to independent socket addresses for resilient execution without impacting performance.
