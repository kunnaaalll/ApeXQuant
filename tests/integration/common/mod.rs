//! Common test utilities for APEX V3 integration tests

use std::time::Duration;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::redis::Redis;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ImageExt};

/// Test environment with all required services
pub struct TestEnvironment {
    pub postgres: ContainerAsync<Postgres>,
    pub redis: ContainerAsync<Redis>,
    pub postgres_url: String,
    pub redis_url: String,
}

impl TestEnvironment {
    /// Start all services for integration tests
    pub async fn start() -> anyhow::Result<Self> {
        // Start PostgreSQL
        let postgres = Postgres::default()
            .with_db_name("apex_test")
            .with_user("apex")
            .with_password("apex_test")
            .start()
            .await?;

        let pg_port = postgres.get_host_port_ipv4(5432).await?;
        let postgres_url = format!(
            "postgres://apex:apex_test@localhost:{}/apex_test",
            pg_port
        );

        // Start Redis
        let redis = Redis::default()
            .start()
            .await?;

        let redis_port = redis.get_host_port_ipv4(6379).await?;
        let redis_url = format!("redis://localhost:{}", redis_port);

        // Wait for services to be ready
        wait_for_postgres(&postgres_url).await?;
        wait_for_redis(&redis_url).await?;

        Ok(Self {
            postgres,
            redis,
            postgres_url,
            redis_url,
        })
    }

    /// Get database pool for testing
    pub async fn db_pool(&self) -> anyhow::Result<sqlx::PgPool> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(&self.postgres_url)
            .await?;

        Ok(pool)
    }

    /// Get Redis client for testing
    pub async fn redis_client(&self) -> anyhow::Result<redis::Client> {
        let client = redis::Client::open(self.redis_url.clone())?;
        Ok(client)
    }

    /// Clean up test data
    pub async fn cleanup(&self) -> anyhow::Result<()> {
        // Clean up Redis
        let client = self.redis_client().await?;
        let mut conn = client.get_multiplexed_async_connection().await?;
        redis::cmd("FLUSHDB").query_async::<_, ()>(&mut conn).await?;

        // Clean up PostgreSQL - truncate all tables
        let pool = self.db_pool().await?;
        sqlx::query("TRUNCATE TABLE sessions, signals, decisions, orders, positions, trades, patterns, lessons, model_weights, market_regimes, performance_snapshots, events, audit_log RESTART IDENTITY CASCADE")
            .execute(&pool)
            .await?;

        Ok(())
    }
}

async fn wait_for_postgres(url: &str) -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(30);

    while start.elapsed() < timeout {
        if sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(url)
            .await
            .is_ok()
        {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    anyhow::bail!("PostgreSQL failed to start within timeout")
}

async fn wait_for_redis(url: &str) -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(30);

    let client = redis::Client::open(url)?;

    while start.elapsed() < timeout {
        if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
            if redis::cmd("PING").query_async::<_, String>(&mut conn).await.is_ok() {
                return Ok(());
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    anyhow::bail!("Redis failed to start within timeout")
}

/// Test fixture that provides common test utilities
pub struct TestFixture {
    pub env: TestEnvironment,
    pub trace_id: String,
}

impl TestFixture {
    pub async fn new() -> anyhow::Result<Self> {
        let env = TestEnvironment::start().await?;
        let trace_id = format!("test-{}", uuid::Uuid::new_v4());

        Ok(Self { env, trace_id })
    }

    /// Run a test with cleanup afterward
    pub async fn run_test<F, Fut, R>(&self, test: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let result = test().await;
        let _ = self.env.cleanup().await;
        result
    }
}
