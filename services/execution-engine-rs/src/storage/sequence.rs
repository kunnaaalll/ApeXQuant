use crate::storage::StorageError;

pub fn validate_sequence_strict(current: u64, next: u64) -> Result<(), StorageError> {
    let expected = current + 1;
    if next != expected {
        Err(StorageError::SequenceViolation {
            expected,
            actual: next,
        })
    } else {
        Ok(())
    }
}
