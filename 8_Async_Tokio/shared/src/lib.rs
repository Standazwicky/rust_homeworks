// shared/src/lib.rs

use serde::{Deserialize, Serialize};
use serde_cbor;
use tracing::instrument;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    Text(String),
    Image(Vec<u8>),
    File(String, Vec<u8>),
    Quit,
    Error(String),
    Login(String),  
    Register(String),
}

// Function to serialize a message
#[instrument]
pub fn serialize_message(message: &MessageType) -> Result<Vec<u8>, SerializationError> {
    serde_cbor::to_vec(&message).map_err(SerializationError::from)
}

// Function to deserialize a message
#[instrument]
pub fn deserialize_message(data: &[u8]) -> Result<MessageType, DeserializationError> {
    serde_cbor::from_slice(&data).map_err(DeserializationError::from)
}

#[derive(Error, Debug)]
pub enum SerializationError {
 #[error("Serialization failed: {0}")]
 Cbor(#[from] serde_cbor::Error),
}

#[derive(Error, Debug)]
pub enum DeserializationError {
 #[error("Deserialization failed: {0}")]
 Cbor(#[from] serde_cbor::Error),
}
