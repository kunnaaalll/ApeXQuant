use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ExecutionConfig {
    pub max_retries: usize,
    pub database_url: String,
    pub grpc_port: u16,
}
