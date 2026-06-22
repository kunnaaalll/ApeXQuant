# Execution Storage Invariants

All persistence interactions strictly enforce the following constraints:

1. **Zero Unsafe**: `#![deny(unsafe_code)]`
2. **Zero Panic/Unwrap/Expect**: The storage layer handles network partition logic, bad JSON parsing, or DB connection limits cleanly using `StorageError`.
3. **Zero Randomness**: UUIDs and IDs are externally generated and deterministically verified.
4. **Zero Business Logic**: The storage repository does not apply policies. It merely provides the interface `Aggregatable` to persist payloads.
5. **Zero Float**: Financial serialization relies purely on `rust_decimal::Decimal`.
