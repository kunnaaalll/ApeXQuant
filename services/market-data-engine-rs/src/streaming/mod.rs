use crate::tick::Tick;
use async_trait::async_trait;

#[async_trait]
pub trait TickStream: Send + Sync {
    async fn connect(&mut self) -> Result<(), String>;
    async fn disconnect(&mut self) -> Result<(), String>;
    async fn next_tick(&mut self) -> Option<Tick>;
}
