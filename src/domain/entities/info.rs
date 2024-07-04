use crate::domain::entities::file::File;
use crate::domain::entities::hashes::Hashes;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use url::Url;

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TorrentInfo {
    #[serde_as(as = "DisplayFromStr")]
    pub announce: Url,
    pub length: i64,
    pub info_hash: [u8; 20],
    pub piece_length: i64,
    pub pieces: Hashes,
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
