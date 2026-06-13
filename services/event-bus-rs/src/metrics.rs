//! Prometheus metrics for Event Bus

use prometheus::{
    Counter, CounterVec, Histogram, HistogramOpts, HistogramVec, IntGauge, Opts, Registry,
};
use std::sync::Arc;
use std::time::Duration;

/// Event Bus metrics collection
#[derive(Clone)]
pub struct EventBusMetrics {
    registry: Arc<Registry>,

    // Event counters
    events_published_total: CounterVec,
    events_stored_total: Counter,
    events_replayed_total: Counter,
    events_dropped_total: Counter,

    // Consumer metrics
    messages_consumed_total: CounterVec,
    messages_acked_total: CounterVec,
    messages_failed_total: CounterVec,
    pending_messages: IntGauge,

    // Latency histograms
    publish_latency: Histogram,
    store_latency: Histogram,
    replay_latency: Histogram,
    consume_latency: HistogramVec,

    // Stream metrics
    stream_length: IntGauge,
    consumer_group_lag: IntGauge,
}

impl EventBusMetrics {
    pub fn new() -> Self {
        let registry = Arc::new(Registry::new());

        let events_published_total = CounterVec::new(
            Opts::new(
                "apex_eventbus_events_published_total",
                "Total number of events published",
            ),
            &["topic", "event_type"],
        )
        .unwrap();

        let events_stored_total = Counter::new(
            "apex_eventbus_events_stored_total",
            "Total number of events stored persistently",
        )
        .unwrap();

        let events_replayed_total = Counter::new(
            "apex_eventbus_events_replayed_total",
            "Total number of events replayed",
        )
        .unwrap();

        let events_dropped_total = Counter::new(
            "apex_eventbus_events_dropped_total",
            "Total number of events dropped due to backpressure",
        )
        .unwrap();

        let messages_consumed_total = CounterVec::new(
            Opts::new(
                "apex_eventbus_messages_consumed_total",
                "Total number of messages consumed by consumers",
            ),
            &["topic", "consumer_group"],
        )
        .unwrap();

        let messages_acked_total = CounterVec::new(
            Opts::new(
                "apex_eventbus_messages_acked_total",
                "Total number of messages acknowledged",
            ),
            &["topic", "consumer_group"],
        )
        .unwrap();

        let messages_failed_total = CounterVec::new(
            Opts::new(
                "apex_eventbus_messages_failed_total",
                "Total number of messages that failed processing",
            ),
            &["topic", "consumer_group", "error_type"],
        )
        .unwrap();

        let pending_messages = IntGauge::new(
            "apex_eventbus_pending_messages",
            "Number of messages pending acknowledgment",
        )
        .unwrap();

        let publish_latency = Histogram::with_opts(
            HistogramOpts::new(
                "apex_eventbus_publish_latency_seconds",
                "Time spent publishing events",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
        )
        .unwrap();

        let store_latency = Histogram::with_opts(
            HistogramOpts::new(
                "apex_eventbus_store_latency_seconds",
                "Time spent storing events",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25]),
        )
        .unwrap();

        let replay_latency = Histogram::with_opts(
            HistogramOpts::new(
                "apex_eventbus_replay_latency_seconds",
                "Time spent replaying events",
            )
            .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]),
        )
        .unwrap();

        let consume_latency = HistogramVec::new(
            HistogramOpts::new(
                "apex_eventbus_consume_latency_seconds",
                "Time spent consuming messages",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
            &["topic", "consumer_group"],
        )
        .unwrap();

        let stream_length = IntGauge::new(
            "apex_eventbus_stream_length",
            "Current number of events in stream",
        )
        .unwrap();

        let consumer_group_lag = IntGauge::new(
            "apex_eventbus_consumer_group_lag",
            "Number of messages behind for consumer group",
        )
        .unwrap();

        // Register all metrics
        registry.register(Box::new(events_published_total.clone())).ok();
        registry.register(Box::new(events_stored_total.clone())).ok();
        registry.register(Box::new(events_replayed_total.clone())).ok();
        registry.register(Box::new(events_dropped_total.clone())).ok();
        registry.register(Box::new(messages_consumed_total.clone())).ok();
        registry.register(Box::new(messages_acked_total.clone())).ok();
        registry.register(Box::new(messages_failed_total.clone())).ok();
        registry.register(Box::new(pending_messages.clone())).ok();
        registry.register(Box::new(publish_latency.clone())).ok();
        registry.register(Box::new(store_latency.clone())).ok();
        registry.register(Box::new(replay_latency.clone())).ok();
        registry.register(Box::new(consume_latency.clone())).ok();
        registry.register(Box::new(stream_length.clone())).ok();
        registry.register(Box::new(consumer_group_lag.clone())).ok();

        Self {
            registry,
            events_published_total,
            events_stored_total,
            events_replayed_total,
            events_dropped_total,
            messages_consumed_total,
            messages_acked_total,
            messages_failed_total,
            pending_messages,
            publish_latency,
            store_latency,
            replay_latency,
            consume_latency,
            stream_length,
            consumer_group_lag,
        }
    }

    /// Record a publish
    pub fn record_publish(&self, latency: Duration) {
        self.publish_latency.observe(latency.as_secs_f64());
    }

    /// Record a batched publish
    pub fn record_batch_publish(&self, count: u64, latency: Duration) {
        self.publish_latency.observe(latency.as_secs_f64());
        // Batch is recorded in caller with topic
    }

    /// Record event published with topic
    pub fn record_event_published(&self, topic: &str, event_type: &str) {
        self.events_published_total
            .with_label_values(&[topic, event_type])
            .inc();
    }

    /// Record event stored
    pub fn record_event_stored(&self) {
        self.events_stored_total.inc();
    }

    /// Record event replayed
    pub fn record_event_replayed(&self, count: usize) {
        self.events_replayed_total.inc_by(count as f64);
    }

    /// Record dropped event
    pub fn record_event_dropped(&self) {
        self.events_dropped_total.inc();
    }

    /// Record message consumed
    pub fn record_message_consumed(&self, topic: &str, consumer_group: &str) {
        self.messages_consumed_total
            .with_label_values(&[topic, consumer_group])
            .inc();
    }

    /// Record message acknowledged
    pub fn record_message_acked(&self, topic: &str, consumer_group: &str) {
        self.messages_acked_total
            .with_label_values(&[topic, consumer_group])
            .inc();
    }

    /// Record message failed
    pub fn record_message_failed(&self, topic: &str, consumer_group: &str, error_type: &str) {
        self.messages_failed_total
            .with_label_values(&[topic, consumer_group, error_type])
            .inc();
    }

    /// Set pending message count
    pub fn set_pending_messages(&self, count: i64) {
        self.pending_messages.set(count);
    }

    /// Record store latency
    pub fn record_store_latency(&self, latency: Duration) {
        self.store_latency.observe(latency.as_secs_f64());
    }

    /// Record replay latency
    pub fn record_replay_latency(&self, latency: Duration) {
        self.replay_latency.observe(latency.as_secs_f64());
    }

    /// Record consume latency
    pub fn record_consume_latency(&self, topic: &str, consumer_group: &str, latency: Duration) {
        self.consume_latency
            .with_label_values(&[topic, consumer_group])
            .observe(latency.as_secs_f64());
    }

    /// Set stream length
    pub fn set_stream_length(&self, length: i64) {
        self.stream_length.set(length);
    }

    /// Set consumer group lag
    pub fn set_consumer_lag(&self, lag: i64) {
        self.consumer_group_lag.set(lag);
    }

    /// Export metrics in Prometheus format
    pub fn export(&self) -> String {
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode_to_string(&metric_families).unwrap_or_default()
    }
}

impl Default for EventBusMetrics {
    fn default() -> Self {
        Self::new()
    }
}
