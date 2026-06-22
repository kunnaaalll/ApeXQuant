use crate::storage::StorageError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AggregateVersion(pub u32);

impl AggregateVersion {
    pub fn next(self) -> Self {
        AggregateVersion(self.0 + 1)
    }

    pub fn validate_transition(current: Self, next: Self) -> Result<(), StorageError> {
        if next <= current {
            Err(StorageError::VersionError)
        } else {
            Ok(())
        }
    }
}
