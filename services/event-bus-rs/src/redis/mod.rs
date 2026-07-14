use deadpool_redis::{Config, Runtime, Pool, Connection};
use anyhow::{Result, Context};

#[derive(Clone)]
pub struct RedisManager {
    pub pool: Pool,
}

impl RedisManager {
    pub async fn connect(url: &str) -> Result<Self> {
        tracing::info!("Connecting to Redis at {}", url);
        let cfg = Config::from_url(url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))
            .context("Failed to create Redis pool")?;
            
        // Test connection
        let _conn = pool.get().await.context("Failed to get Redis connection from pool")?;
        
        Ok(Self { pool })
    }
    
    pub async fn get_connection(&self) -> Result<Connection> {
        self.pool.get().await.context("Failed to get connection")
    }
}
