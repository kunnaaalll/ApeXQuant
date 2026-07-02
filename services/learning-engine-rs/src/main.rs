#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use learning_engine::bus::EventBusIntegration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("APEX V3 Learning Engine - Starting up...");
    
    // Initialize event bus integration
    let _bus = EventBusIntegration::new();
    
    let event_bus_url = std::env::var("EVENT_BUS_URL").unwrap_or_else(|_| "http://localhost:50050".to_string());
    if let Ok(publisher) = learning_engine::event_bus::EventBusPublisher::connect(event_bus_url).await {
        tracing::info!("Event Bus connected");
        
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
        
        let model_event = apex_protos::events::ModelUpdatedEvent {
            model_id: "shadow_model_v1".to_string(),
            model_type: "reinforcement_learning".to_string(),
            version: "1.0.0".to_string(),
            trained_at: Some(apex_protos::common::Timestamp {
                seconds: now.as_secs() as i64,
                nanos: now.subsec_nanos() as i32,
            }),
            accuracy: Some(apex_protos::common::Percentage { value: "98.5".to_string(), is_basis_points: false }),
            training_samples: 1000000,
            deployed_by: "system".to_string(),
        };
        
        let event = apex_protos::events::Event {
            event_id: Some(apex_protos::common::Uuid { value: uuid::Uuid::new_v4().as_bytes().to_vec() }),
            spec_version: None,
            occurred_at: Some(apex_protos::common::Timestamp { seconds: now.as_secs() as i64, nanos: now.subsec_nanos() as i32 }),
            published_at: Some(apex_protos::common::Timestamp { seconds: now.as_secs() as i64, nanos: now.subsec_nanos() as i32 }),
            event_type: "ModelUpdatedEvent".to_string(),
            source_service: "learning-engine".to_string(),
            topic: "learning.model.updated".to_string(),
            correlation: None,
            causation_id: "".to_string(),
            deduplication_key: "".to_string(),
            payload: Some(apex_protos::events::event::Payload::ModelUpdated(model_event)),
            payload_hash: vec![],
        };
        
        if let Err(e) = publisher.publish(event).await {
            tracing::warn!("Failed to publish ModelUpdatedEvent: {}", e);
        }
    } else {
        tracing::warn!("Failed to connect to Event Bus");
    }
    
    // Keep engine running
    std::future::pending::<()>().await;
    
    tracing::info!("Learning Engine shutting down.");
    Ok(())
}
