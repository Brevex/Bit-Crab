use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;
use crate::domain::peers::Peers;

#[derive(Debug)]
pub struct TorrentInfo
{
    pub announce: Option<String>,
    pub length: Option<i64>,
    pub piece_length: Option<i64>,
    pub pieces: Option<Vec<String>>,
    pub info_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackerRequest
{
    pub peer_id: String,
    pub port: u16,
    pub uploaded: usize,
    pub downloaded: usize,
    pub left: usize,
    pub compact: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackerResponse
{
    pub peers: Option<Peers>,
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
    SingleFile
    {
        length: usize,
    },
    MultiFile
    {
        files: Vec<File>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File
{
    pub length: usize,
    pub path: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Hashes(pub Vec<[u8; 20]>);

impl<'de> Deserialize<'de> for Hashes
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HashesVisitor;
        impl<'de> Visitor<'de> for HashesVisitor
        {
            type Value = Hashes;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result
            {
                formatter.write_str("a byte string whose length is a multiple of 20")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.len() % 20 != 0 {
                    return Err(E::custom(format!("length is {}", v.len())));
                }
                Ok(Hashes(
                    v.chunks_exact(20)
                        .map(|slice| slice.try_into().expect("length is 20"))
                        .collect(),
                ))
            }
        }
        deserializer.deserialize_bytes(HashesVisitor)
    }
}

impl Serialize for Hashes
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let single_slice: Vec<u8> = self.0.iter().flatten().copied().collect();
        serializer.serialize_bytes(&single_slice)
    }
}
