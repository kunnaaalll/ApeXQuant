use apex_protos::events::Event;
use anyhow::Result;

pub struct Dispatcher {
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn dispatch(&self, _event: Event) -> Result<()> {
        // Core dispatch logic will interact with router and publisher
        Ok(())
    }
}
