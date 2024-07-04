use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Peer {
    pub id: Uuid,
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct Handshake {
    pub protocol: String,
    pub reserved: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Self {
            protocol: "BitTorrent protocol".to_string(),
            reserved: [0; 8],
            info_hash,
            peer_id,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(68);
        bytes.push(self.protocol.len() as u8);
        bytes.extend_from_slice(self.protocol.as_bytes());
        bytes.extend_from_slice(&self.reserved);
        bytes.extend_from_slice(&self.info_hash);
        bytes.extend_from_slice(&self.peer_id);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 68 {
            return Err(HandshakeError::InvalidLength.into());
        }

        let protocol_len = bytes[0] as usize;
        if &bytes[1..1 + protocol_len] != b"BitTorrent protocol" {
            return Err(HandshakeError::InvalidProtocol.into());
        }

        let mut info_hash = [0; 20];
        info_hash.copy_from_slice(&bytes[28..48]);

        let mut peer_id = [0; 20];
        peer_id.copy_from_slice(&bytes[48..68]);

        Ok(Self {
            protocol: "BitTorrent protocol".to_string(),
            reserved: [0; 8],
            info_hash,
            peer_id,
        })
    }
}

#[derive(Debug)]
pub enum HandshakeError {
    InvalidLength,
    InvalidProtocol,
}

impl fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for HandshakeError {}
