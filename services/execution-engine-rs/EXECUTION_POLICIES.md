# Execution Policies Implementation

## Policies
- **Market**: Executes immediately at whatever price is available.
- **Limit**: Executes only at specified price or better.
- **IOC (Immediate Or Cancel)**: Fills whatever is available immediately; cancels the rest.
- **FOK (Fill Or Kill)**: Fills completely immediately, or cancels entirely if not possible. Cannot be partially filled.
- **GTC (Good Till Cancelled)**: Stays active until explicitly cancelled.

State transitions are strongly typed and statically enforced. Illegal state transitions return `PolicyError`.
