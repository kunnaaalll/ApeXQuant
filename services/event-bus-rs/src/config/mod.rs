use anyhow::{Context, Result};
use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub bind_address: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
        let redis_url = env::var("REDIS_URL").context("REDIS_URL must be set")?;
        let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "[::]:50050".to_string());

        Ok(Self {
            database_url,
            redis_url,
            bind_address,
        })
    }
}
