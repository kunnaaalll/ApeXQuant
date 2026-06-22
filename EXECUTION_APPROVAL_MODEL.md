# Execution Approval Model

Institutional workflow ensures promotion cannot bypass verification states.

## State Progression
- **NotReady:** Default base state. System has seen failures recently.
- **Monitoring:** Reached after 10 consecutive perfect parity checks.
- **Candidate:** Reached after 50 consecutive perfect parity checks.
- **Approved:** Reached after 100 consecutive perfect parity checks.

## Demotion Mechanics
Failure breaks the streak immediately (sets it to `0`).
State drops are strictly sequential:
- `Approved` drops to `Candidate`
- `Candidate` drops to `Monitoring`
- `Monitoring` drops to `NotReady`

This prevents a single isolated outlier from instantly triggering an absolute shutdown if the system was previously ultra-stable, acting as an exponential moving average buffer for catastrophic overrides.
