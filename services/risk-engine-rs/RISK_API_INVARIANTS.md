# Risk API Invariants

1. **No Business Logic**: The API layer shall not contain any business logic or calculate risk matrices. It merely constructs and delegates transport queries.
2. **Deterministic Responses**: The API layer must return identically shaped responses for identically requested states. 
3. **Panic-Free Handling**: Any invalid input or missing metadata (like headers) must never cause a panic. It must smoothly map to standard `tonic::Status` representations.
4. **Safety Verification**: The entire crate enforces `#![deny(unsafe_code)]`.
5. **No Randomness**: The API must never generate random data or use random IDs internally.
6. **Observability First**: All incoming RPC requests must be logged via `tracing`, and request duration and error counts must be captured using `metrics`.
