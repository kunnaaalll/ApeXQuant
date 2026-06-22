# Execution Repository Architecture

The `ExecutionRepository` combines storage and rebuilder mechanics, orchestrating database transactions without implementing business algorithms itself.

## Duties:
- **`save()`**: Executes one atomic transaction that batches new events, evaluates snapshot frequency logic to selectively persist new snapshots, and commits.
- **`load()`**: Dispatches read queries for the latest snapshot, then dispatches reads for trailing events, delegating construction to the `ExecutionEventRebuilder`.
