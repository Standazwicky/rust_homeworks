// shared/src/lib.rs

use serde::{Deserialize, Serialize};
use serde_cbor;
use thiserror::Error;
use tracing::instrument;

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

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn test_serialize_deserialize_text_message() {
      let message = MessageType::Text("Příliš žluťoučký kůň úpěl ďábelské ódy!".to_string());
      let serialized = serialize_message(&message).unwrap();
      let deserialized: MessageType = deserialize_message(&serialized).unwrap();
      
      if let MessageType::Text(text) = deserialized {
         assert_eq!(text, "Příliš žluťoučký kůň úpěl ďábelské ódy!")   
      } else {
         panic!("Deserialized message is not of type Text");   
      }
  }
  
  #[test]
  fn test_serialize_deserialize_image_message() {
     let message = MessageType::Image(vec![1, 2, 3, 4, 5]);
     let serialized = serialize_message(&message).unwrap();
     let deserialized: MessageType = deserialize_message(&serialized).unwrap();
     
     if let MessageType::Image(data) = deserialized {
        assert_eq!(data, vec![1, 2, 3, 4, 5]);
     } else {
         panic!("Deserialized message is not of type Image");   
     }
  }
  
  #[test]
  fn test_serialize_deserialize_file_message() {
      let message = MessageType::File("test.txt".to_string(), vec![1, 2, 3, 4, 5, 6]);
      let serialized = serialize_message(&message).unwrap();
      let deserialized: MessageType = deserialize_message(&serialized).unwrap();
      
      if let MessageType::File(filename, data) = deserialized {
         assert_eq!(filename, "test.txt");
         assert_eq!(data, vec![1, 2, 3, 4, 5, 6]);
      } else {
         panic!("Deserialized message is not of type File");   
      }
  }
}
