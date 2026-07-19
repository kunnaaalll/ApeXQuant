use anyhow::{Context, Result};
use apex_protos::events::Event;
use prost::Message;

pub fn serialize_event(event: &Event) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    event
        .encode(&mut buf)
        .context("Failed to serialize event")?;
    Ok(buf)
}

pub fn deserialize_event(data: &[u8]) -> Result<Event> {
    Event::decode(data).context("Failed to deserialize event")
}
