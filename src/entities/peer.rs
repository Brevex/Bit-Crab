use crate::entities::torrent::Torrent;
use crate::utils::extract_torrent_metadata::generate_peer_id;

use getset::Getters;
use reqwest::Url;
use std::net::IpAddr;
use urlencoding::encode_binary;

#[derive(Getters, Clone, Debug)]
pub struct Peer
{
    #[get = "pub"]
    ip: IpAddr,
    #[get = "pub"]
    port: u16,
}

impl Peer
{
    pub fn new(ip: IpAddr, port: u16) -> Self
    {
        Self { ip, port }
    }
}

#[derive(Getters, Clone, Debug)]
pub struct TrackerResponse
{
    #[get = "pub"]
    interval: i64,
    #[get = "pub"]
    peers: Vec<Peer>,
}

impl TrackerResponse
{
    pub fn new(interval: i64, peers: Vec<Peer>) -> Self
    {
        Self { interval, peers }
    }
}

#[derive(Getters, Clone, Debug)]
pub struct TrackerRequest
{
    #[get = "pub"]
    tracker_url: Url,
    #[get = "pub"]
    info_hash: [u8; 20],
    #[get = "pub"]
    peer_id: String,
    #[get = "pub"]
    port: u16,
    #[get = "pub"]
    uploaded: i64,
    #[get = "pub"]
    downloaded: i64,
    #[get = "pub"]
    left: i64,
    #[get = "pub"]
    compact: u8,
}

impl TrackerRequest
{
    pub fn new(torrent: &Torrent) -> Self
    {
        Self
        {
            tracker_url: torrent.announce().clone(),
            info_hash: *torrent.info_hash(),
            peer_id: generate_peer_id(),
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left: *torrent.info().length(),
            compact: 1,
        }
    }

    pub fn build_url(&self) -> String
    {
        format!(
            "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}",
            self.tracker_url,
            encode_binary(&self.info_hash),
            encode_binary(self.peer_id.as_bytes()),
            self.port,
            self.uploaded,
            self.downloaded,
            self.left,
            self.compact
        )
    }
}