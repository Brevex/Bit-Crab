use crate::utils::extract_torrent_metadata::generate_peer_id;
use getset::Getters;

#[derive(Getters, Clone, Debug)]
pub struct Handshake
{
    #[get = "pub"]
    protocol_str: String,
    #[get = "pub"]
    reserved: [u8; 8],
    #[get = "pub"]
    info_hash: [u8; 20],
    #[get = "pub"]
    peer_id: String,
}

impl Handshake
{
    pub fn new(info_hash: [u8; 20]) -> Self
    {
        Self
        {
            protocol_str: "BitTorrent protocol".to_string(),
            reserved: [0; 8],
            info_hash,
            peer_id: generate_peer_id(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes = Vec::new();
        bytes.push(self.protocol_str.len() as u8);
        bytes.extend_from_slice(self.protocol_str.as_bytes());
        bytes.extend_from_slice(&self.reserved);
        bytes.extend_from_slice(&self.info_hash);
        bytes.extend_from_slice(self.peer_id.as_bytes());
        bytes
    }
}
