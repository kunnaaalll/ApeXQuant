# Connection State Machine

A broker connection follows a deterministic path defined by `ConnectionState`:

```rust
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Degraded,
    Reconnecting,
    Failed,
}
```

## Supported Transitions
- `Disconnected` -> `Connecting`
- `Connecting` -> `Connected` | `Failed`
- `Connected` -> `Degraded` | `Disconnected`
- `Degraded` -> `Connected` | `Reconnecting` | `Disconnected`
- `Reconnecting` -> `Connected` | `Failed`
- `Failed` -> `Disconnected` | `Connecting`

Illegal jumps are strictly blocked at compile/runtime using a state verification algorithm, returning `Result::Err`.
