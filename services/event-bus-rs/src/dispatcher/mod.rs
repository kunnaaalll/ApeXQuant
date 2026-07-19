use anyhow::Result;
use apex_protos::events::Event;

pub struct Dispatcher {}

impl Dispatcher {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Dispatcher {
    pub async fn dispatch(&self, _event: Event) -> Result<()> {
        // Core dispatch logic will interact with router and publisher
        Ok(())
    }
}
