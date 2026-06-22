use crate::storage::StorageError;
use serde_json::Value;

pub struct ExecutionSerializer;

impl ExecutionSerializer {
    pub fn serialize<T: serde::Serialize>(state: &T) -> Result<Value, StorageError> {
        let json = serde_json::to_value(state)?;
        Ok(json)
    }

    pub fn deserialize<T: serde::de::DeserializeOwned>(json: Value) -> Result<T, StorageError> {
        let state = serde_json::from_value(json)?;
        Ok(state)
    }
}
