# Risk Event Sourcing Strategy

## Design Goal
All state mutations inside the Risk Engine must be the result of a discrete, replayable event. This guarantees that historical portfolio states can be reconstructed precisely, simplifying debugging and creating audit-proof behavior.

## Event Envelope
Every domain event is wrapped in a `PortfolioEventWrapper` and stored as an `EventRecord`.

The structure enforces:
- `aggregate_id`: The ID of the portfolio/entity.
- `sequence`: Strict sequential ordering.
- `version`: Used for optimistic concurrency control.
- `timestamp`: UTC offset time for determinism.

## Immutability
Once an event is appended, it is never modified or deleted. 
If an erroneous state occurs, compensating events must be issued. The database explicitly restricts updates to the `risk_events` table.
