use apex_protos::events::{
    event_bus_service_server::EventBusService,
    PublishRequest, PublishResponse, SubscribeRequest, EventBatch,
    AckRequest, AckResponse, StreamStatsRequest, StreamStats, Event,
};
use tonic::{Request, Response, Status};
use tokio::sync::{mpsc, broadcast};
use tokio_stream::wrappers::ReceiverStream;
use crate::storage::EventStore;
use crate::router::SequenceManager;
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
            let sequenced_event = self.sequencer.sequence_event(event).await
                .map_err(|e| Status::internal(format!("Sequencing error: {}", e)))?;

            self.store.store_event(&sequenced_event).await
                .map_err(|e| Status::internal(format!("Storage error: {}", e)))?;

            if let Some(event_id) = &sequenced_event.event_id {
                let id_str = String::from_utf8_lossy(&event_id.value).into_owned();
                published_ids.push(id_str);
            }

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
        
        let consumer_group = req.consumer_group.clone();
        let topics = req.topics.clone();
        let store = self.store.clone();
        
        let mut start_time = Utc::now();
        
        if !consumer_group.is_empty() {
            let mut min_time: Option<sqlx::types::chrono::DateTime<Utc>> = None;
            for topic in &topics {
                if let Ok(Some((_, occurred_at))) = store.get_subscriber_offset(&consumer_group, topic).await {
                    min_time = Some(match min_time {
                        Some(t) => if occurred_at < t { occurred_at } else { t },
                        None => occurred_at,
                    });
                }
            }
            if let Some(t) = min_time {
                start_time = t;
            } else {
                start_time = calculate_start_time(&req.start_from);
            }
        } else {
            start_time = calculate_start_time(&req.start_from);
        }
        
        tokio::spawn(async move {
            // Replay historical events
            if let Ok(historical_events) = store.load_events_by_topic(&topics, start_time).await {
                for event in historical_events {
                    let batch = EventBatch {
                        events: vec![event],
                        next_position: String::new(),
                        has_more: false,
                        total_available: 1,
                    };
                    if tx.send(Ok(batch)).await.is_err() {
                        return;
                    }
                }
            }

            // Stream live events
            while let Ok(event) = rx_broadcast.recv().await {
                if topics.contains(&event.topic) {
                    let occurred_dt = event.occurred_at.as_ref()
                        .and_then(|t| sqlx::types::chrono::DateTime::<Utc>::from_timestamp(t.seconds, t.nanos as u32))
                        .ok_or_else(|| Status::internal("Missing occurred_at timestamp in broadcasted event"));
                    
                    if let Ok(occurred_dt) = occurred_dt {
                        if occurred_dt > start_time {
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
        let consumer_group = req.consumer_group.clone();
        
        let mut acknowledged = 0;
        let mut failed = 0;
        let mut moved_to_dlq = 0;

        if !consumer_group.is_empty() {
            // Process successful acknowledgments
            for id_str in &req.event_ids {
                if let Ok(event_id) = uuid::Uuid::from_str(id_str) {
                    if let Ok(Some(event)) = self.store.get_event(event_id).await {
                        let occurred_dt = event.occurred_at.as_ref()
                            .and_then(|t| sqlx::types::chrono::DateTime::<Utc>::from_timestamp(t.seconds, t.nanos as u32));
                            
                        if let Some(dt) = occurred_dt {
                            if self.store.update_subscriber_offset(&consumer_group, &event.topic, event_id, dt).await.is_ok() {
                                acknowledged += 1;
                            } else {
                                failed += 1;
                            }
                        } else {
                            failed += 1;
                        }
                    } else {
                        failed += 1;
                    }
                } else {
                    failed += 1;
                }
            }

            // Process failed messages -> send to DLQ
            for entry in &req.failed {
                if let Ok(event_id) = uuid::Uuid::from_str(&entry.event_id) {
                    let mut payload_bytes = vec![];
                    let mut topic = "unknown".to_string();
                    if let Ok(Some(event)) = self.store.get_event(event_id).await {
                        topic = event.topic.clone();
                        let _ = prost::Message::encode(&event, &mut payload_bytes);
                    }

                    let err_msg = entry.error.as_ref().map(|e| e.message.clone()).unwrap_or_else(|| String::from("Unknown error"));

                    if self.store.move_to_dlq(
                        &consumer_group,
                        &topic,
                        Some(event_id),
                        &payload_bytes,
                        &entry.reason,
                        &err_msg
                    ).await.is_ok() {
                        moved_to_dlq += 1;
                    }
                }
            }
        }

        Ok(Response::new(AckResponse {
            result: Some(apex_protos::common::Result {
                ok: true,
                error: None,
            }),
            acknowledged,
            failed,
            moved_to_dlq,
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

fn calculate_start_time(pos: &Option<apex_protos::events::StreamPosition>) -> sqlx::types::chrono::DateTime<Utc> {
    use apex_protos::events::StartPosition;
    let from_enum = pos.as_ref()
        .and_then(|p| p.position.as_ref())
        .map(|p| match p {
            apex_protos::events::stream_position::Position::From(val) => *val,
            _ => 0,
        })
        .unwrap_or(0);

    let now = Utc::now();
    let epoch = sqlx::types::chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap_or(now);

    if from_enum == StartPosition::Earliest as i32 {
        epoch
    } else if from_enum == StartPosition::NowMinusHour as i32 {
        now - chrono::Duration::hours(1)
    } else if from_enum == StartPosition::NowMinusDay as i32 {
        now - chrono::Duration::days(1)
    } else if from_enum == StartPosition::NowMinusWeek as i32 {
        now - chrono::Duration::weeks(1)
    } else {
        now
    }
}
