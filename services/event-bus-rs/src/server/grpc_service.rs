use apex_protos::events::{
    event_bus_service_server::EventBusService,
    PublishRequest, PublishResponse, SubscribeRequest, EventBatch,
    AckRequest, AckResponse, StreamStatsRequest, StreamStats, Event,
};
use tonic::{Request, Response, Status};
use tokio::sync::{mpsc, broadcast};
use tokio_stream::wrappers::ReceiverStream;
use crate::storage::EventStore;
use crate::routing::SequenceManager;
use sqlx::types::chrono::Utc;
use std::str::FromStr;

pub struct EventBusServiceImpl {
    store: EventStore,
    sequencer: SequenceManager,
    broadcaster: broadcast::Sender<Event>,
}

impl EventBusServiceImpl {
    pub fn new(store: EventStore, sequencer: SequenceManager) -> Self {
        let (broadcaster, _) = broadcast::channel(10000);
        Self { store, sequencer, broadcaster }
    }
}

#[tonic::async_trait]
impl EventBusService for EventBusServiceImpl {
    async fn publish(
        &self,
        request: Request<PublishRequest>,
    ) -> Result<Response<PublishResponse>, Status> {
        let req = request.into_inner();
        let mut published_ids = Vec::new();

        for event in req.events {
            // Sequence the event
            let sequenced_event = self.sequencer.sequence_event(event).await
                .map_err(|e| Status::internal(format!("Sequencing error: {}", e)))?;

            // Store the event based on durability requirements
            // For now, we always persist it to the EventStore
            self.store.store_event(&sequenced_event).await
                .map_err(|e| Status::internal(format!("Storage error: {}", e)))?;

            if let Some(event_id) = &sequenced_event.event_id {
                let id_str = String::from_utf8_lossy(&event_id.value).into_owned();
                published_ids.push(id_str);
            }

            // Notify subscribers
            let _ = self.broadcaster.send(sequenced_event);
        }

        Ok(Response::new(PublishResponse {
            result: Some(apex_protos::common::Result {
                ok: true,
                error: None,
            }),
            published_ids,
        }))
    }

    type SubscribeStream = ReceiverStream<Result<EventBatch, Status>>;

    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let req = request.into_inner();
        let (tx, rx) = mpsc::channel(100);
        let mut rx_broadcast = self.broadcaster.subscribe();
        
        let topics = req.topics.clone();
        
        tokio::spawn(async move {
            while let Ok(event) = rx_broadcast.recv().await {
                if topics.contains(&event.topic) {
                    let batch = EventBatch {
                        events: vec![event],
                        next_position: String::new(),
                        has_more: false,
                        total_available: 1,
                    };
                    if tx.send(Ok(batch)).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn ack(
        &self,
        request: Request<AckRequest>,
    ) -> Result<Response<AckResponse>, Status> {
        let req = request.into_inner();

        // We assume req.event_ids are valid UUIDs that were successfully processed.
        if let Some(last_id_str) = req.event_ids.last() {
            if let Ok(last_id) = uuid::Uuid::from_str(last_id_str) {
                // We'd update offset here. We need the topic and occurred_at
            }
        }
        
        Ok(Response::new(AckResponse {
            result: Some(apex_protos::common::Result {
                ok: true,
                error: None,
            }),
            acknowledged: req.event_ids.len() as u32,
            failed: req.failed.len() as u32,
            moved_to_dlq: 0, // This could map to DLQ logic for req.failed
        }))
    }

    async fn get_stream_stats(
        &self,
        request: Request<StreamStatsRequest>,
    ) -> Result<Response<StreamStats>, Status> {
        let topic = request.into_inner().topic;
        let stats = self.store.get_stream_stats(&topic).await
            .map_err(|e| Status::internal(format!("Failed to get stream stats: {}", e)))?;
            
        let oldest = stats.2.map(|t| apex_protos::common::Timestamp {
            seconds: t.timestamp(),
            nanos: t.timestamp_subsec_nanos() as i32,
        });
        
        let newest = stats.3.map(|t| apex_protos::common::Timestamp {
            seconds: t.timestamp(),
            nanos: t.timestamp_subsec_nanos() as i32,
        });

        Ok(Response::new(StreamStats {
            topic,
            total_events: stats.0 as u64,
            byte_size: stats.1 as u64,
            oldest_event: oldest,
            newest_event: newest,
            consumers: vec![],
        }))
    }
}
