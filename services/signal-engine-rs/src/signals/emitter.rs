//! Signal emitter for broadcasting signals

use crate::event_bus::EventBusPublisher;
use crate::signals::SignalResult;
use apex_protos::common::{Price, Timestamp, Uuid};
use apex_protos::events::{event::Payload, Event, SignalDetectedEvent};
use std::sync::Arc;

/// Signal event emitter
pub struct SignalEmitter {
    event_bus: Option<Arc<EventBusPublisher>>,
}

impl SignalEmitter {
    pub fn new(event_bus: Arc<EventBusPublisher>) -> Self {
        Self {
            event_bus: Some(event_bus),
        }
    }

    pub async fn emit_signal(&self, signal: &SignalResult) -> crate::error::Result<()> {
        tracing::info!(
            "Emitting Signal: {:?} {} @ {:?} (Conf: {:.2})",
            signal.direction,
            signal.symbol,
            signal.entry_price,
            signal.confidence
        );
        if let Some(bus) = &self.event_bus {
            let signal_event = SignalDetectedEvent {
                signal_id: signal.signal_id.clone(),
                symbol: signal.symbol.clone(),
                timeframe: Some(apex_protos::common::Timeframe {
                    value: 15,
                    unit: apex_protos::common::TimeUnit::Minute as i32,
                }), // Assuming M15 for now based on generator
                strategy_id: "core_smc_mtf".to_string(),
                pattern_type: if signal.patterns.is_empty() {
                    "unknown".to_string()
                } else {
                    signal.patterns.join(",")
                },
                suggested_side: match signal.direction {
                    crate::signals::result::SignalDirection::Long => {
                        apex_protos::common::TradeSide::Buy as i32
                    }
                    crate::signals::result::SignalDirection::Short => {
                        apex_protos::common::TradeSide::Sell as i32
                    }
                    crate::signals::result::SignalDirection::Neutral => {
                        apex_protos::common::TradeSide::Unspecified as i32
                    }
                },
                entry_zone_low: Some(Price {
                    value: signal.entry_price.to_string(),
                    digits: 0,
                    currency: "USD".to_string(),
                }),
                entry_zone_high: Some(Price {
                    value: signal.entry_price.to_string(),
                    digits: 0,
                    currency: "USD".to_string(),
                }),
                confluence_score: signal.confluence_score,
                confluence_factors: vec![signal.regime.clone()], // Can map detailed factors here
                valid_until: None,
                raw_detections: std::collections::HashMap::new(),
            };

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let event = Event {
                event_id: Some(Uuid {
                    value: uuid::Uuid::new_v4().as_bytes().to_vec(),
                }),
                spec_version: None,
                occurred_at: Some(Timestamp {
                    seconds: now.as_secs() as i64,
                    nanos: now.subsec_nanos() as i32,
                }),
                published_at: Some(Timestamp {
                    seconds: now.as_secs() as i64,
                    nanos: now.subsec_nanos() as i32,
                }),
                event_type: "SignalDetectedEvent".to_string(),
                source_service: "signal-engine".to_string(),
                topic: "signals.detected".to_string(),
                correlation: None,
                causation_id: "".to_string(),
                deduplication_key: "".to_string(),
                payload: Some(Payload::SignalDetected(signal_event)),
                payload_hash: vec![],
            };

            bus.publish(event).await.map_err(|e| {
                crate::error::SignalEngineError::Internal(format!("Event bus error: {}", e))
            })?;
        }
        Ok(())
    }
}
