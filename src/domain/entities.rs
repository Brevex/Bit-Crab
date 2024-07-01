use url::Url;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TorrentInfo {
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub announce: Option<Url>,
    pub length: Option<i64>,
    pub info_hash: Option<String>,
    pub piece_length: Option<i64>,
    pub pieces: Option<Vec<String>>,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Torrent {
    #[serde_as(as = "DisplayFromStr")]
    pub announce: Url,
    pub info: Info,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Info {
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    pub pieces: Hashes,
    #[serde(flatten)]
    pub keys: Keys,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Keys {
    SingleFile { length: usize },
    MultiFile { files: Vec<File> },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    pub length: usize,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Hashes(#[serde(with = "serde_bytes")] pub Vec<u8>);

impl Hashes {
    pub fn to_hash_vec(&self) -> Vec<[u8; 20]> {
        self.0.chunks_exact(20)
            .map(|chunk| chunk.try_into().unwrap())
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Peer {
    pub id: Uuid,
    pub ip: String,
    pub port: u16,
}
