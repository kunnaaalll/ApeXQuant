# Performance Certification Criteria

This document outlines the states and definitions of the certification progression.

## Certification States

1. **NotReady**: Code compiles but fails one or more critical invariants.
2. **Experimental**: Passes determinism and zero panics. Safe for internal benching.
3. **ShadowCertified**: Matches the TypeScript outputs above 99.9%. Allowed to run alongside production without gating external systems.
4. **ProductionCertified**: Parity, Determinism, Replay, and Benchmark pass. Safe for actual user queries.
5. **InstitutionalCertified**: Has passed Monte Carlo edge collapse simulations and max stress boundaries without falling over. Safe for institutional routing.
