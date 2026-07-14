use anyhow::Result;
use sqlx::PgPool;
use crate::nats::NatsManager;
use crate::redis::RedisManager;

#[derive(Clone)]
pub struct HealthChecker {
    db_pool: PgPool,
    nats: NatsManager,
    redis: RedisManager,
}

impl HealthChecker {
    pub fn new(db_pool: PgPool, nats: NatsManager, redis: RedisManager) -> Self {
        Self { db_pool, nats, redis }
    }

    pub async fn check_postgres(&self) -> bool {
        sqlx::query("SELECT 1").execute(&self.db_pool).await.is_ok()
    }

    pub async fn check_nats(&self) -> bool {
        self.nats.client.connection_state() == async_nats::connection::State::Connected
    }

    pub async fn check_redis(&self) -> bool {
        self.redis.get_connection().await.is_ok()
    }
    
    pub async fn is_healthy(&self) -> bool {
        self.check_postgres().await && self.check_nats().await && self.check_redis().await
    }
}
