# Failure & Recovery Operations

This subsystem controls post-anomaly system recovery.

## Failure Tracking
Monitors broker errors, routing failures, and timeouts. Outputs a failure score clamped at `0-100`.

## Cooldown Requirements
Requires multiple stable cycles, healthy latency, and successful fills before recovery begins.

## Sequential Recovery
Recovery is strictly sequential. An immediate state reset back to locked/frozen occurs upon any new failure mid-recovery.
