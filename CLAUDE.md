# APEX V3

## Quick Commands

```bash
# Start all services
(cd infrastructure/docker && docker-compose up -d)

# Run database migrations
pnpm db:migrate

# Build all Rust services
cargo build --workspace

# Run tests
pnpm test
cargo test --workspace

# Check service health
curl http://localhost:8080/health
```

## Architecture

See `FOUNDATION_IMPLEMENTATION_PLAN.md` for complete architecture documentation.

Key principles:
- Event-driven with immutable events
- gRPC for service-to-service communication
- Protobuf contracts are source of truth
- PostgreSQL for persistence, Redis for streams
- Every service exposes /health, /metrics
