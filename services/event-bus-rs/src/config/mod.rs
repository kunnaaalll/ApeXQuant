use std::env;
use anyhow::{Result, Context};

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub nats_url: String,
    pub redis_url: String,
    pub bind_address: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .context("DATABASE_URL must be set")?;
        let nats_url = env::var("NATS_URL")
            .context("NATS_URL must be set")?;
        let redis_url = env::var("REDIS_URL")
            .context("REDIS_URL must be set")?;
        let bind_address = env::var("BIND_ADDRESS")
            .unwrap_or_else(|_| "[::]:50050".to_string());
            
        Ok(Self {
            database_url,
            nats_url,
            redis_url,
            bind_address,
        })
    }
}
