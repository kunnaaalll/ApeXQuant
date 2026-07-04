use apex_protos::events::Event;
use sqlx::PgPool;
use anyhow::Result;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum ReplayMode {
    Realtime,
    TenX,
    HundredX,
    ThousandX,
}

pub struct ReplayEngine {
    pool: PgPool,
}

impl ReplayEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn replay_by_topic(&self, topic: &str, mode: ReplayMode) -> Result<tokio::sync::mpsc::Receiver<Event>> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        let pool = self.pool.clone();
        let topic_clone = topic.to_string();
        
        tokio::spawn(async move {
            use sqlx::Row;
            let rows = sqlx::query(
                r#"
                SELECT payload, occurred_at
                FROM events
                WHERE topic = $1
                ORDER BY occurred_at ASC
                "#
            )
            .bind(topic_clone)
            .fetch_all(&pool)
            .await;

            if let Ok(rows) = rows {
                let mut prev_time: Option<chrono::DateTime<chrono::Utc>> = None;
                for r in rows {
                    let payload: Vec<u8> = r.get("payload");
                    let occurred_at: chrono::DateTime<chrono::Utc> = r.get("occurred_at");
                    if let Ok(event) = prost::Message::decode(&payload[..]) {
                        if let Some(prev) = prev_time {
                            let diff = occurred_at.signed_duration_since(prev);
                            let sleep_ms = match mode {
                                ReplayMode::Realtime => diff.num_milliseconds(),
                                ReplayMode::TenX => diff.num_milliseconds() / 10,
                                ReplayMode::HundredX => diff.num_milliseconds() / 100,
                                ReplayMode::ThousandX => diff.num_milliseconds() / 1000,
                            };
                            if sleep_ms > 0 {
                                tokio::time::sleep(tokio::time::Duration::from_millis(sleep_ms as u64)).await;
                            }
                        }
                        prev_time = Some(occurred_at);
                        if tx.send(event).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Ok(rx)
    }

    pub async fn replay_by_time_range(
        &self, 
        start: chrono::DateTime<chrono::Utc>, 
        end: chrono::DateTime<chrono::Utc>, 
        mode: ReplayMode
    ) -> Result<tokio::sync::mpsc::Receiver<Event>> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        let pool = self.pool.clone();
        
        tokio::spawn(async move {
            use sqlx::Row;
            let rows = sqlx::query(
                r#"
                SELECT payload, occurred_at
                FROM events
                WHERE occurred_at >= $1 AND occurred_at <= $2
                ORDER BY occurred_at ASC
                "#
            )
            .bind(start)
            .bind(end)
            .fetch_all(&pool)
            .await;

            if let Ok(rows) = rows {
                let mut prev_time: Option<chrono::DateTime<chrono::Utc>> = None;
                for r in rows {
                    let payload: Vec<u8> = r.get("payload");
                    let occurred_at: chrono::DateTime<chrono::Utc> = r.get("occurred_at");
                    if let Ok(event) = prost::Message::decode(&payload[..]) {
                        if let Some(prev) = prev_time {
                            let diff = occurred_at.signed_duration_since(prev);
                            let sleep_ms = match mode {
                                ReplayMode::Realtime => diff.num_milliseconds(),
                                ReplayMode::TenX => diff.num_milliseconds() / 10,
                                ReplayMode::HundredX => diff.num_milliseconds() / 100,
                                ReplayMode::ThousandX => diff.num_milliseconds() / 1000,
                            };
                            if sleep_ms > 0 {
                                tokio::time::sleep(tokio::time::Duration::from_millis(sleep_ms as u64)).await;
                            }
                        }
                        prev_time = Some(occurred_at);
                        if tx.send(event).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Ok(rx)
    }
}
