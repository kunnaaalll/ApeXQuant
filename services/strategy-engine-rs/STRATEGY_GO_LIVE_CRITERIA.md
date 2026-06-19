# Go-Live Criteria

Before a shadow-mode strategy can progress, it must pass extremely strict continuous `ExactMatch` streaks.

## Staging Requirements

- `NotReady`: 0 to 99 consecutive exact matches
- `Monitoring`: 100 to 999 consecutive exact matches
- `Candidate`: 1,000 to 9,999 consecutive exact matches
- `Approved`: 10,000+ consecutive exact matches

## Demotions

Demotions are severe but step-wise to prevent accidental jumps:

- `Approved` -> `Candidate`
- `Candidate` -> `Monitoring`
- `Monitoring` -> `NotReady`

State skipping is permanently forbidden by design.
