use crate::shadow::{ShadowEvent, ShadowSnapshot};

pub trait ShadowStorage {
    type Error;

    fn append_snapshot(&self, snapshot: &ShadowSnapshot) -> Result<(), Self::Error>;
    fn load_snapshots(&self) -> Result<Vec<ShadowSnapshot>, Self::Error>;

    fn append_event(&self, event: &ShadowEvent) -> Result<(), Self::Error>;
    fn load_events(&self) -> Result<Vec<ShadowEvent>, Self::Error>;
}
