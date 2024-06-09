// shared/src/lib.rs

use serde::{Deserialize, Serialize};
use serde_cbor;
use tracing::instrument;

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    Text(String),
    Image(Vec<u8>),
    File(String, Vec<u8>),
    Quit,
}

// Function to serialize a message
#[instrument]
pub fn serialize_message(message: &MessageType) -> Vec<u8> {
    serde_cbor::to_vec(&message).unwrap()
}

// Function to deserialize a message
#[instrument]
pub fn deserialize_message(data: &[u8]) -> Result<MessageType, Box<dyn std::error::Error>> {
    let message = serde_cbor::from_slice(&data)?;
    Ok(message)
}
