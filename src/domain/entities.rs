use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TorrentInfo
{
    pub announce: Option<String>,
    pub length: Option<i64>,
    pub piece_length: Option<i64>,
    pub pieces: Option<Vec<String>>,
    pub info_hash: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Torrent
{
    pub announce: String,
    pub info: Info,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Info
{
    pub name: String,

    #[serde(rename = "piece length")]
    pub piece_length: usize,
    pub pieces: Hashes,

    #[serde(flatten)]
    pub keys: Keys,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Keys
{
    SingleFile {
        length: usize,
    },
    MultiFile {
        files: Vec<File>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File
{
    pub length: usize,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Hashes(#[serde(with = "serde_bytes")] pub Vec<u8>);

impl Hashes
{
    pub fn to_hash_vec(&self) -> Vec<[u8; 20]>
    {
        self.0.chunks_exact(20)
            .map(|chunk| chunk.try_into().unwrap())
            .collect()
    }
}
