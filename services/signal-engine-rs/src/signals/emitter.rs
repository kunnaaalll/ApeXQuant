//! Signal emitter for broadcasting signals

use std::sync::Arc;
use crate::signals::SignalResult;
use crate::event_bus::EventBusPublisher;
use apex_protos::events::{Event, event::Payload, SignalDetectedEvent};
use apex_protos::common::{Price, Timestamp, Uuid};

/// Signal event emitter
pub struct SignalEmitter {
    event_bus: Option<Arc<EventBusPublisher>>,
}

impl SignalEmitter {
    pub fn new(event_bus: Option<Arc<EventBusPublisher>>) -> Self {
        Self { event_bus }
    }

    pub fn emit(&self, signal: &SignalResult) {
        // Placeholder for signal emission logic
        tracing::info!(
            "Signal: {:?} {} @ {:?}",
            signal.direction,
            signal.symbol,
            signal.confidence
        );
        if let Some(bus) = &self.event_bus {
            let signal_event = SignalDetectedEvent {
                signal_id: signal.signal_id.clone(),
                symbol: signal.symbol.clone(),
                timeframe: Some(apex_protos::common::Timeframe { value: 1, unit: apex_protos::common::TimeUnit::Minute as i32 }),
                strategy_id: "default_strategy".to_string(),
                pattern_type: "unknown".to_string(),
                suggested_side: match signal.direction {
                    crate::signals::result::SignalDirection::Long => apex_protos::common::TradeSide::Buy as i32,
                    crate::signals::result::SignalDirection::Short => apex_protos::common::TradeSide::Sell as i32,
                    crate::signals::result::SignalDirection::Neutral => apex_protos::common::TradeSide::Unspecified as i32,
                },
                entry_zone_low: Some(Price { value: signal.entry_price.to_string(), digits: 0, currency: "USD".to_string() }),
                entry_zone_high: Some(Price { value: signal.entry_price.to_string(), digits: 0, currency: "USD".to_string() }),
                confluence_score: signal.confluence_score,
                confluence_factors: vec![],
                valid_until: None,
                raw_detections: std::collections::HashMap::new(),
            };

            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
            let event = Event {
                event_id: Some(Uuid { value: uuid::Uuid::new_v4().as_bytes().to_vec() }),
                spec_version: None,
                occurred_at: Some(Timestamp { seconds: now.as_secs() as i64, nanos: now.subsec_nanos() as i32 }),
                published_at: Some(Timestamp { seconds: now.as_secs() as i64, nanos: now.subsec_nanos() as i32 }),
                event_type: "SignalDetectedEvent".to_string(),
                source_service: "signal-engine".to_string(),
                topic: "signals.detected".to_string(),
                correlation: None,
                causation_id: "".to_string(),
                deduplication_key: "".to_string(),
                payload: Some(Payload::SignalDetected(signal_event)),
                payload_hash: vec![],
            };

            let bus = bus.clone();
            tokio::spawn(async move {
                if let Err(e) = bus.publish(event).await {
                    tracing::warn!("Failed to publish SignalDetectedEvent: {}", e);
                }
            });
        }
    }
}
