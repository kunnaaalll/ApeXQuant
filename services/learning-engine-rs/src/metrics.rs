use metrics::{counter, gauge, histogram};
use std::time::Duration;

pub struct LearningEngineMetrics;

impl LearningEngineMetrics {
    pub fn record_lesson_processed(success: bool) {
        if success {
            counter!("apex_learning_lessons_processed_total", "status" => "success").increment(1);
        } else {
            counter!("apex_learning_lessons_processed_total", "status" => "failure").increment(1);
        }
    }

    pub fn record_adaptation_latency(duration: Duration) {
        histogram!("apex_learning_adaptation_latency_seconds").record(duration.as_secs_f64());
    }

    pub fn update_memory_usage(bytes: u64) {
        gauge!("apex_learning_memory_usage_bytes").set(bytes as f64);
    }

    pub fn record_feature_discovered(feature_type: &str) {
        counter!("apex_learning_features_discovered_total", "type" => feature_type.to_string()).increment(1);
    }

    pub fn record_recommendation_generated(strategy: &str) {
        counter!("apex_learning_recommendations_total", "strategy" => strategy.to_string()).increment(1);
    }
}
