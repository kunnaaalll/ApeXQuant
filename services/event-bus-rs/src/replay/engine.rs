use apex_protos::events::Event;
use sqlx::PgPool;
use anyhow::Result;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum ReplayMode {
    Realtime,
    TenX,
    HundredX,
    ThousandX,
    Immediate, // Deterministic fast-forward mode
}

pub struct ReplayEngine {
    pool: PgPool,
}

impl ReplayEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn replay_by_topic(&self, topic: &str, _mode: ReplayMode) -> Result<tokio::sync::mpsc::Receiver<Event>> {
        let (tx, rx) = tokio::sync::mpsc::channel(1000);
        let pool = self.pool.clone();
        let topic_clone = topic.to_string();
        
        tokio::spawn(async move {
            use sqlx::Row;
            let rows = sqlx::query(
                r#"
                SELECT payload
                FROM events
                WHERE topic = $1
                ORDER BY occurred_at ASC
                "#
            )
            .bind(topic_clone)
            .fetch_all(&pool)
            .await;

            if let Ok(rows) = rows {
                for r in rows {
                    let payload: Vec<u8> = r.get("payload");
                    if let Ok(event) = prost::Message::decode(&payload[..]) {
                        // Deliver as fast as possible without arbitrary sleep (deterministic)
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
        _mode: ReplayMode
    ) -> Result<tokio::sync::mpsc::Receiver<Event>> {
        let (tx, rx) = tokio::sync::mpsc::channel(1000);
        let pool = self.pool.clone();
        
        tokio::spawn(async move {
            use sqlx::Row;
            let rows = sqlx::query(
                r#"
                SELECT payload
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
                for r in rows {
                    let payload: Vec<u8> = r.get("payload");
                    if let Ok(event) = prost::Message::decode(&payload[..]) {
                        // Deliver as fast as possible without arbitrary sleep (deterministic)
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
