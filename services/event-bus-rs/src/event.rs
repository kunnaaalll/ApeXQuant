//! Event types and utilities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Core event structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    /// Global unique event ID
    pub id: Uuid,

    /// Event metadata
    pub metadata: EventMetadata,

    /// Event payload
    #[serde(flatten)]
    pub payload: EventPayload,
}

/// Event metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventMetadata {
    /// Schema version
    pub spec_version: String,

    /// UTC timestamp when event occurred
    pub occurred_at: DateTime<Utc>,

    /// UTC timestamp when event was published
    pub published_at: DateTime<Utc>,

    /// Fully qualified event type name
    pub event_type: String,

    /// Service that emitted the event
    pub source_service: String,

    /// Event bus topic/destination
    pub topic: String,

    /// Correlation context for distributed tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation: Option<CorrelationContext>,

    /// ID of event that caused this one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<String>,

    /// Key for deduplication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deduplication_key: Option<String>,
}

/// Correlation context for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CorrelationContext {
    pub trace_id: Uuid,
    pub span_id: Uuid,
    pub sampled: bool,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub baggage: HashMap<String, String>,
}

/// Event payload (type-erased JSON)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_type")]
pub enum EventPayload {
    #[serde(rename = "signal.detected")]
    SignalDetected(SignalDetectedPayload),

    #[serde(rename = "signal.validated")]
    SignalValidated(SignalValidatedPayload),

    #[serde(rename = "signal.rejected")]
    SignalRejected(SignalRejectedPayload),

    #[serde(rename = "execution.filled")]
    ExecutionFilled(ExecutionFilledPayload),

    #[serde(rename = "position.opened")]
    PositionOpened(PositionOpenedPayload),

    #[serde(rename = "position.closed")]
    PositionClosed(PositionClosedPayload),

    #[serde(rename = "risk.breach")]
    RiskBreach(RiskBreachPayload),

    #[serde(rename = "system.health")]
    SystemHealth(SystemHealthPayload),

    #[serde(rename = "market.data")]
    MarketData(MarketDataPayload),

    // Fallback for unknown event types
    #[serde(other)]
    Raw(serde_json::Value),
}

// Payload types - these would be generated from protobuf

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignalDetectedPayload {
    pub signal_id: String,
    pub symbol: String,
    pub timeframe: String,
    pub direction: String,
    pub confluence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignalValidatedPayload {
    pub signal_id: String,
    pub approved: bool,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignalRejectedPayload {
    pub signal_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionFilledPayload {
    pub order_id: String,
    pub position_id: String,
    pub fill_price: String,
    pub fill_volume: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PositionOpenedPayload {
    pub position_id: String,
    pub symbol: String,
    pub side: String,
    pub entry_price: String,
    pub volume: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PositionClosedPayload {
    pub position_id: String,
    pub exit_price: String,
    pub pnl: String,
    pub close_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskBreachPayload {
    pub limit_type: String,
    pub current_value: String,
    pub limit_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemHealthPayload {
    pub service: String,
    pub healthy: bool,
    pub metrics: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketDataPayload {
    pub symbol: String,
    pub timeframe: String,
    pub timestamp: DateTime<Utc>,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
}

impl Event {
    /// Create a new event
    pub fn new(
        event_type: impl Into<String>,
        topic: impl Into<String>,
        source_service: impl Into<String>,
        payload: EventPayload,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            metadata: EventMetadata {
                spec_version: "1.0".to_string(),
                occurred_at: now,
                published_at: now,
                event_type: event_type.into(),
                source_service: source_service.into(),
                topic: topic.into(),
                correlation: None,
                causation_id: None,
                deduplication_key: None,
            },
            payload,
        }
    }

    /// Create with correlation context
    pub fn with_correlation(mut self, correlation: CorrelationContext) -> Self {
        self.metadata.correlation = Some(correlation);
        self
    }

    /// Create with causation ID
    pub fn with_causation(mut self, causation_id: impl Into<String>) -> Self {
        self.metadata.causation_id = Some(causation_id.into());
        self
    }

    /// Create with deduplication key
    pub fn with_deduplication_key(mut self, key: impl Into<String>) -> Self {
        self.metadata.deduplication_key = Some(key.into());
        self
    }

    /// Get stream key for Redis
    pub fn stream_key(&self) -> String {
        format!("events:{}", self.metadata.topic)
    }

    /// Serialize to Redis stream fields
    pub fn to_stream_fields(&self) -> crate::Result<Vec<(String, String)>> {
        let mut fields = vec![
            ("id".to_string(), self.id.to_string()),
            ("spec_version".to_string(), self.metadata.spec_version.clone()),
            ("occurred_at".to_string(), self.metadata.occurred_at.to_rfc3339()),
            ("published_at".to_string(), self.metadata.published_at.to_rfc3339()),
            ("event_type".to_string(), self.metadata.event_type.clone()),
            ("source_service".to_string(), self.metadata.source_service.clone()),
            ("topic".to_string(), self.metadata.topic.clone()),
        ];

        if let Some(ref corr) = self.metadata.correlation {
            fields.push(("trace_id".to_string(), corr.trace_id.to_string()));
            fields.push(("span_id".to_string(), corr.span_id.to_string()));
            fields.push(("sampled".to_string(), corr.sampled.to_string()));
        }

        if let Some(ref causation) = self.metadata.causation_id {
            fields.push(("causation_id".to_string(), causation.clone()));
        }

        if let Some(ref dedup) = self.metadata.deduplication_key {
            fields.push(("deduplication_key".to_string(), dedup.clone()));
        }

        // Serialize payload as JSON
        let payload_json = serde_json::to_string(&self.payload)?;
        fields.push(("payload".to_string(), payload_json));

        Ok(fields)
    }

    /// Deserialize from Redis stream fields
    pub fn from_stream_fields(id: &str, fields: &HashMap<String, String>) -> crate::Result<Self> {
        let event = Self {
            id: fields.get("id")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(Uuid::new_v4),
            metadata: EventMetadata {
                spec_version: fields.get("spec_version").cloned().unwrap_or_else(|| "1.0".to_string()),
                occurred_at: fields.get("occurred_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
                published_at: fields.get("published_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
                event_type: fields.get("event_type").cloned().unwrap_or_default(),
                source_service: fields.get("source_service").cloned().unwrap_or_default(),
                topic: fields.get("topic").cloned().unwrap_or_default(),
                correlation: None, // TODO: Parse correlation context
                causation_id: fields.get("causation_id").cloned(),
                deduplication_key: fields.get("deduplication_key").cloned(),
            },
            payload: fields.get("payload")
                .and_then(|p| serde_json::from_str(p).ok())
                .unwrap_or_else(|| EventPayload::Raw(serde_json::json!({}))),
        };

        Ok(event)
    }

    /// Validate the event
    pub fn validate(&self) -> crate::Result<()> {
        if self.metadata.event_type.is_empty() {
            return Err(crate::EventBusError::Validation(
                "event_type cannot be empty".to_string()
            ));
        }

        if self.metadata.topic.is_empty() {
            return Err(crate::EventBusError::Validation(
                "topic cannot be empty".to_string()
            ));
        }

        if self.metadata.source_service.is_empty() {
            return Err(crate::EventBusError::Validation(
                "source_service cannot be empty".to_string()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = Event::new(
            "test.event",
            "test.topic",
            "test-service",
            EventPayload::Raw(serde_json::json!({"key": "value"})),
        );

        assert_eq!(event.metadata.event_type, "test.event");
        assert_eq!(event.metadata.topic, "test.topic");
        assert_eq!(event.metadata.source_service, "test-service");
    }

    #[test]
    fn test_event_serialization() {
        let event = Event::new(
            "test.event",
            "test.topic",
            "test-service",
            EventPayload::Raw(serde_json::json!({"key": "value"})),
        );

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("test.event"));
        assert!(json.contains("test.topic"));

        let deserialized: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(event.id, deserialized.id);
    }
}
