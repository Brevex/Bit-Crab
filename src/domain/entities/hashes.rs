use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Hashes(#[serde(with = "serde_bytes")] pub Vec<u8>);

impl Hashes
{
    pub fn to_hash_vec(&self) -> Vec<[u8; 20]>
    {
        self.0.chunks_exact(20).map(|chunk| {
            chunk.try_into().expect("Chunk length should be 20 bytes")
        }).collect()
    }
}