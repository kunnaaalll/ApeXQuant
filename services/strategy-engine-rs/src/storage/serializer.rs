use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum SerializerError {
    SerializationFailed(String),
    DeserializationFailed(String),
}

impl std::fmt::Display for SerializerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializerError::SerializationFailed(err) => write!(f, "Serialization failed: {}", err),
            SerializerError::DeserializationFailed(err) => {
                write!(f, "Deserialization failed: {}", err)
            }
        }
    }
}

impl std::error::Error for SerializerError {}

pub struct Serializer;

impl Serializer {
    pub fn serialize<T: Serialize>(value: &T) -> Result<serde_json::Value, SerializerError> {
        serde_json::to_value(value).map_err(|e| SerializerError::SerializationFailed(e.to_string()))
    }

    pub fn deserialize<T: for<'de> Deserialize<'de>>(
        value: serde_json::Value,
    ) -> Result<T, SerializerError> {
        serde_json::from_value(value)
            .map_err(|e| SerializerError::DeserializationFailed(e.to_string()))
    }
}
