# Risk Go-Live Criteria

1. Complete sequential transition through `NotReady` -> `Monitoring` -> `Candidate` -> `Approved`.
2. 100,000 consecutive identical matches achieved in testing without divergence.
3. Zero critical mismatches on circuit breaker, stress assessment, and recommendation states within the Candidate phase.
4. Guaranteed non-interference demonstrated (Shadow mode only observes and compares).
