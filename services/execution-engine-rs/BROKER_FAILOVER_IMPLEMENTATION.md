# Broker Failover Implementation

Failover is strictly managed through the `FailoverState` enum.

## Sequence Enforcement
The allowed transition paths are explicitly defined:
1. `Healthy` -> `Warning`
2. `Warning` -> `Failover`
3. `Failover` -> `Recovery`
4. `Recovery` -> `Warning`
5. `Warning` -> `Healthy`

### Forbidden Transitions
A direct transition from `Failover` to `Healthy` is **illegal** and will yield a `BrokerError::InvalidStateTransition`. A system returning from a failover event must undergo `Recovery` to re-synchronize order books, positions, and history before becoming completely `Healthy`.
