# Execution Certification Criteria

Certification progresses sequentially:
`NotCertified` -> `Candidate` -> `Certified`

Rules:
- To become a Candidate, the system must pass all validation gates (Parity, Determinism, Replay, Stress, Benchmark).
- To become Certified, the system must pass again while in Candidate state.
- A failure resets the state or drops it by one level.
- `Rejected` state cannot jump straight to `Certified`.
