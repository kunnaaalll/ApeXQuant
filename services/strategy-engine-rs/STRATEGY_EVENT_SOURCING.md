# Strategy Event Sourcing

The Strategy Engine captures every structural change as a domain event, wrapped within `StrategyEventWrapper`.

## Wrapped Types
- `HealthEvent`
- `ConfidenceEvent`
- `DriftEvent`
- `AllocationEvent`
- `RecommendationEvent`
- `DegradationEvent`
- `MetaEvent`
- `ClusterEvent`
- `ContextEvent`
- `ValidationEvent`
- `ShadowEvent`

All events are strictly serialized as JSON objects and saved linearly to `strategy_events`.
Rebuilding state leverages the `Aggregatable` trait, utilizing `apply_event()` on streams.
