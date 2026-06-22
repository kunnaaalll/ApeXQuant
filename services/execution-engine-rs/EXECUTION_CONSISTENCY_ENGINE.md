# Execution Consistency Engine

The Consistency Engine sits directly above the event-sourcing layer evaluating the structural health of execution sequences. 

## Invariants Evaluated
1. **Event Count Consistency**: Database counts align with memory boundaries.
2. **Sequence Consistency**: Strictly contiguous IDs.
3. **Snapshot Alignment**: Snapshots exactly mirror target sequence checkpoints.
4. **Version Consistency**: Forward-only protocol versioning.
5. **Aggregate ID Consistency**: Identifiers strictly tethered to intended partition tables.

## Statuses
- `Healthy`
- `Warning`
- `Broken`

If `Broken` is encountered, trading stops for the compromised partition.
