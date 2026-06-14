# APEX V3 Portfolio Storage Engine Invariants

The Portfolio Storage Engine enforces strict invariants to ensure institutional-grade reliability, security, and performance.

## 1. Immutability Invariants
- **Append-Only Logs:** Event records are strictly append-only. Under no circumstances is an event ever mutated or deleted once committed to the database.
- **Snapshot Integrity:** Snapshots represent an immutable state at a specific timestamp. They cannot be altered retrospectively.

## 2. Determinism Invariants
- **Reproducibility:** Given an empty starting state and an ordered sequence of events from $1$ to $N$, the reconstructed state MUST be byte-for-byte and logically identical to the original snapshot at version $N$.
- **No Hidden State:** The storage engine guarantees that absolutely no business state is persisted outside of the defined `EventRecord` or `SnapshotRecord` structures.
- **Strict Ordering:** Events for a specific aggregate are strictly ordered by a monotonic `version` integer.

## 3. Transactional Invariants
- **Atomicity:** When an operation generates both an event and a snapshot, both must be committed to the database within a single, atomic SQL transaction. If one fails, both must fail.
- **Optimistic Concurrency:** Concurrent writes must be isolated using optimistic locking. If two writers attempt to commit version $N$ for the same aggregate, exactly one will succeed, and the other will receive a conflict error and must rollback and retry.

## 4. Performance Targets
The storage engine must meet the following performance invariants under nominal load:
- **Write Latency:** $< 5$ ms for appending a single event.
- **Read Latency:** $< 3$ ms for retrieving the latest real-time snapshot.
- **P99 Latency:** $< 20$ ms for complex operations (e.g., event appending + snapshot updating).
- **Memory Leaks:** $0$
- **Unsafe Code:** $0$
- **Panics:** $0$ (All errors must be handled and returned as `Result<T, E>`)

## 5. Security & Audit Invariants
- **Traceability:** Every event must have a timestamp and a unique `id` (UUIDv4).
- **Metadata Context:** Every event must include metadata outlining the context of the change (e.g., origin system, correlation IDs).
