//! Message encoding/decoding for network transport

use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::error::NetworkError;
use crate::message::NetworkMessage;

/// Maximum message size (16 MB).
const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024;

/// Length prefix size (4 bytes).
const LENGTH_PREFIX_SIZE: usize = 4;

/// Codec for encoding/decoding network messages.
///
/// Wire format: [length: u32 BE] [bincode-encoded message]
#[derive(Debug, Default)]
pub struct MessageCodec {
    /// Partial decode state
    decode_state: DecodeState,
}

#[derive(Debug, Default)]
enum DecodeState {
    #[default]
    ReadingLength,
    ReadingPayload {
        length: usize,
    },
}

impl MessageCodec {
    /// Creates a new codec.
    pub fn new() -> Self {
        Self::default()
    }

    /// Encodes a message to bytes (standalone function).
    pub fn encode_message(msg: &NetworkMessage) -> Result<Vec<u8>, NetworkError> {
        let payload = bincode::serialize(msg)
            .map_err(|e| NetworkError::Codec(format!("serialize error: {}", e)))?;

        if payload.len() > MAX_MESSAGE_SIZE {
            return Err(NetworkError::Codec(format!(
                "message too large: {} bytes (max {})",
                payload.len(),
                MAX_MESSAGE_SIZE
            )));
        }

        let mut buf = Vec::with_capacity(LENGTH_PREFIX_SIZE + payload.len());
        buf.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        buf.extend_from_slice(&payload);
        Ok(buf)
    }

    /// Decodes a message from bytes (standalone function).
    pub fn decode_message(data: &[u8]) -> Result<NetworkMessage, NetworkError> {
        if data.len() < LENGTH_PREFIX_SIZE {
            return Err(NetworkError::Codec("data too short for length prefix".into()));
        }

        let length = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;

        if length > MAX_MESSAGE_SIZE {
            return Err(NetworkError::Codec(format!(
                "message length {} exceeds max {}",
                length, MAX_MESSAGE_SIZE
            )));
        }

        let expected_len = LENGTH_PREFIX_SIZE + length;
        if data.len() < expected_len {
            return Err(NetworkError::Codec(format!(
                "incomplete message: have {} bytes, need {}",
                data.len(),
                expected_len
            )));
        }

        let payload = &data[LENGTH_PREFIX_SIZE..expected_len];
        bincode::deserialize(payload)
            .map_err(|e| NetworkError::Codec(format!("deserialize error: {}", e)))
    }
}

impl Decoder for MessageCodec {
    type Item = NetworkMessage;
    type Error = NetworkError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            match &self.decode_state {
                DecodeState::ReadingLength => {
                    if src.len() < LENGTH_PREFIX_SIZE {
                        return Ok(None);
                    }

                    let length = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;

                    if length > MAX_MESSAGE_SIZE {
                        return Err(NetworkError::Codec(format!(
                            "message length {} exceeds max {}",
                            length, MAX_MESSAGE_SIZE
                        )));
                    }

                    src.advance(LENGTH_PREFIX_SIZE);
                    self.decode_state = DecodeState::ReadingPayload { length };
                }
                DecodeState::ReadingPayload { length } => {
                    let length = *length;
                    if src.len() < length {
                        return Ok(None);
                    }

                    let payload = src.split_to(length);
                    self.decode_state = DecodeState::ReadingLength;

                    let msg: NetworkMessage = bincode::deserialize(&payload)
                        .map_err(|e| NetworkError::Codec(format!("deserialize error: {}", e)))?;

                    return Ok(Some(msg));
                }
            }
        }
    }
}

impl Encoder<NetworkMessage> for MessageCodec {
    type Error = NetworkError;

    fn encode(&mut self, item: NetworkMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let payload = bincode::serialize(&item)
            .map_err(|e| NetworkError::Codec(format!("serialize error: {}", e)))?;

        if payload.len() > MAX_MESSAGE_SIZE {
            return Err(NetworkError::Codec(format!(
                "message too large: {} bytes (max {})",
                payload.len(),
                MAX_MESSAGE_SIZE
            )));
        }

        dst.reserve(LENGTH_PREFIX_SIZE + payload.len());
        dst.put_u32(payload.len() as u32);
        dst.put_slice(&payload);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::PROTOCOL_VERSION;

    #[test]
    fn test_encode_decode_roundtrip() {
        let msg = NetworkMessage::Hello {
            version: PROTOCOL_VERSION,
            peer_id: [1u8; 32],
            genesis_hash: [2u8; 32],
            public_key: [3u8; 64],
        };

        let encoded = MessageCodec::encode_message(&msg).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();

        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_codec_streaming() {
        let mut codec = MessageCodec::new();
        let mut buf = BytesMut::new();

        let msg1 = NetworkMessage::Ping(12345);
        let msg2 = NetworkMessage::GetPeers;

        // Encode both messages
        codec.encode(msg1.clone(), &mut buf).unwrap();
        codec.encode(msg2.clone(), &mut buf).unwrap();

        // Decode first
        let decoded1 = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(msg1, decoded1);

        // Decode second
        let decoded2 = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(msg2, decoded2);

        // Nothing left
        assert!(codec.decode(&mut buf).unwrap().is_none());
    }

    #[test]
    fn test_partial_decode() {
        let mut codec = MessageCodec::new();
        let msg = NetworkMessage::Pong(99999);
        let encoded = MessageCodec::encode_message(&msg).unwrap();

        // Feed bytes one at a time
        let mut buf = BytesMut::new();
        for (i, byte) in encoded.iter().enumerate() {
            buf.extend_from_slice(&[*byte]);
            let result = codec.decode(&mut buf).unwrap();
            if i < encoded.len() - 1 {
                assert!(result.is_none());
            } else {
                assert_eq!(result, Some(msg.clone()));
            }
        }
    }

    #[test]
    fn test_message_too_large() {
        let large_data = vec![0u8; MAX_MESSAGE_SIZE + 1];
        let msg = NetworkMessage::Transactions(vec![crate::message::SerializableTransaction {
            nonce: 0,
            to: None,
            value: [0u8; 32],
            data: large_data,
            signature: [0u8; 65],
        }]);

        let result = MessageCodec::encode_message(&msg);
        assert!(result.is_err());
    }
}
