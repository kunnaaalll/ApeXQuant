use crate::events::StreamingEvent;
use tokio::sync::broadcast;

pub struct MarketDispatcher {
    sender: broadcast::Sender<StreamingEvent>,
}

impl Default for MarketDispatcher {
    fn default() -> Self {
        Self::new(1024)
    }
}

impl MarketDispatcher {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn publish(&self, event: StreamingEvent) -> Result<usize, broadcast::error::SendError<StreamingEvent>> {
        self.sender.send(event)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<StreamingEvent> {
        self.sender.subscribe()
    }
}
