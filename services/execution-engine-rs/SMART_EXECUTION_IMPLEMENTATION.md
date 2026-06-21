# Smart Execution Implementation

The Smart Execution Engine dictates HOW orders are executed, not WHY or WHAT.
It handles routing, priorities, and urgency, remaining fully isolated from trading logic.

## Urgency
- **Patient**: Passive execution, primarily limit orders on best venue.
- **Balanced**: Seeks a balance between speed and price.
- **Aggressive**: Sacrifices some slippage for faster execution.
- **Emergency**: Will cross spread immediately to fill.

## Priority
- **Low**: Executed after all other orders.
- **Normal**: Default FIFO priority.
- **High**: Gets allocated routing limits first.
- **Critical**: Skips standard queues, preempts other orders.

## Routing State
- **Primary**: Default path for an execution.
- **Secondary**: Backup when primary is slow or degrading.
- **Fallback**: Emergency venue used only when critical.
