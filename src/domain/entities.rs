use serde::{Deserialize, Serialize};
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
    pub interval: usize,
    pub complete: Option<usize>,
    pub incomplete: Option<usize>,
    pub peers: Option<Peers>,
}