//! Signal emitter for broadcasting signals

use crate::signals::SignalResult;

/// Signal event emitter
pub struct SignalEmitter;

impl SignalEmitter {
    pub fn new() -> Self {
        Self
    }

    pub fn emit(&self, signal: &SignalResult) {
        // Placeholder for signal emission logic
        tracing::info!(
            "Signal: {:?} {} @ {:?}",
            signal.direction,
            signal.symbol,
            signal.entry_zone
        );
    }
}

impl Default for SignalEmitter {
    fn default() -> Self {
        Self::new()
    }
}
