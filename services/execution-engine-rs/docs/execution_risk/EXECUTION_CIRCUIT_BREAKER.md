# Execution Circuit Breaker

Maintains the global `ExecutionProtectionState`.

## States
- `Normal`
- `Warning`
- `Restricted`
- `Critical`
- `Frozen`

## Transition Rules
- Immediate escalation to higher danger states is permitted.
- Recovery to lower danger states must be sequential (e.g., Frozen -> Critical -> Restricted -> Warning -> Normal).
- Jumping sequentially is strictly enforced; invalid jumps return `ExecutionError::IllegalTransition`.
