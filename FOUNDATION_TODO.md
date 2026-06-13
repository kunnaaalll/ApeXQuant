# APEX V3 Foundation - Remaining Tasks

## Infrastructure (Ready to Build)

- [ ] Run `cargo build` in `services/event-bus-rs` and fix any compilation errors
- [ ] Run `pnpm install` from root to verify workspace setup
- [ ] Run `docker-compose build` from `infrastructure/docker` to verify Dockerfiles
- [ ] Create placeholder `src/main.rs` files for all Rust services (with health endpoints)
- [ ] Add generated protobuf Rust code to `shared/apex-protos/src`

## Phase 2 - Signal Engine V1 (Next Milestone)

- [ ] Port TypeScript SMC detection to Rust
- [ ] Implement candlestick data structures
- [ ] Implement Order Block detection
- [ ] Implement FVG detection
- [ ] Implement BOS/CHoCH detection
- [ ] Implement multi-timeframe synchronization
- [ ] Connect signal engine to event bus
- [ ] Write integration tests for signal detection

## Phase 3 - Full Pipeline

- [ ] Risk engine integration
- [ ] AI council integration
- [ ] Execution engine with MT5 bridge
- [ ] Position management
- [ ] Portfolio tracking

## Documentation to Complete

- [ ] Rust service README files
- [ ] API documentation
- [ ] Deployment guide
- [ ] Operations runbook
