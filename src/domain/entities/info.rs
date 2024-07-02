use url::Url;
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use serde::{Deserialize, Serialize};
use crate::domain::entities::file::File;
use crate::domain::entities::hashes::Hashes;

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TorrentInfo
{
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub announce: Option<Url>,
    pub length: Option<i64>,
    pub info_hash: Option<[u8; 20]>,
    pub piece_length: Option<i64>,
    pub pieces: Option<Hashes>,
    pub info: Option<Info>,
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
    SingleFile { length: usize },
    MultiFile { files: Vec<File> },
}