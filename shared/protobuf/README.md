# APEX V3 Protobuf Definitions

This directory contains the Protocol Buffer schema definitions for all APEX V3 services.

## Design Principles

1. **Versioning**: All messages include explicit version fields for backward compatibility
2. **Immutability**: All events are append-only, never updated in place
3. **Strong Typing**: All identifiers use custom wrapper types to prevent confusion
4. **Audit Trail**: Every message includes correlation ID and timestamp

## File Organization

- `common.proto` - Shared types, wrappers, and base messages
- `signal.proto` - Signal detection and market analysis
- `risk.proto` - Risk assessment and limits
- `execution.proto` - Order routing and execution
- `position.proto` - Position tracking and management
- `portfolio.proto` - Portfolio-level analytics
- `analytics.proto` - Performance metrics and reports
- `learning.proto` - Model weights and training data
- `events.proto` - Event envelope and streaming protocol

## Generation

### Rust
```bash
cargo build -p apex-protos
```

### TypeScript
```bash
pnpm proto:generate
```

## Version Compatibility

- Fields must never be renumbered
- New fields should use reserved tags above 1000
- Deprecated fields must be marked with `(deprecated) = true`
- Breaking changes require a new `.proto` file with incremented version
