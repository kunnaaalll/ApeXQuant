# Execution Rebuilder

The `ExecutionEventRebuilder` ensures absolute fidelity between live and reconstructed state.

It works in these phases:
1. Load aggregate snapshot.
2. Load any events whose sequence numbers follow the snapshot's sequence count.
3. Iteratively invoke `apply_event` over the subset of event stream.
4. Yield the perfectly restored execution aggregate state.

If a sequence gap happens during reconstruction, it fails immediately to prevent corrupted operation.
