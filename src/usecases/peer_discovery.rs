use crate::domain::entities::info::TorrentInfo;
use crate::domain::entities::peer::Peer;
use crate::domain::errors::TorrentError;
use crate::usecases::utils::generate_peer_id::generate_peer_id;
use anyhow::{Context, Result};
use reqwest::Client;
use serde_bencode::value::Value;
use std::collections::HashMap;
use urlencoding::encode_binary;

pub async fn request_peers(torrent_info: &TorrentInfo) -> Result<Vec<Peer>> {
    let tracker_url = &torrent_info.announce;
    let info_hash_encoded = get_info_hash_encoded(&torrent_info)?;
    let peer_id = generate_peer_id();
    let peer_id_encoded = encode_binary(&peer_id);
    let length = torrent_info.length;

    let url = format!(
        "{}?info_hash={}&peer_id={}&port=6881&uploaded=0&downloaded=0&left={}&compact=1",
        tracker_url, info_hash_encoded, peer_id_encoded, length as u64
    );

    let response = send_tracker_request(&url).await?;
    let peers_compact = extract_peers_compact(&response)?;
    let peers = parse_peers(peers_compact);

    Ok(peers)
}

fn get_info_hash_encoded(torrent_info: &TorrentInfo) -> Result<String> {
    let info_hash = torrent_info.info_hash;

    Ok(info_hash
        .iter()
        .map(|byte| format!("%{:02x}", byte))
        .collect::<String>())
}

async fn send_tracker_request(url: &str) -> Result<bytes::Bytes> {
    let client = Client::new();
    let response = client.get(url).send().await?.bytes().await?;

    println!("Tracker response: {:?}", response);
    Ok(response)
}

fn extract_peers_compact(response: &bytes::Bytes) -> Result<Vec<u8>> {
    let decoded_response: HashMap<String, Value> =
        serde_bencode::from_bytes(&response).context("Failed to decode bencode response")?;

    match decoded_response.get("peers") {
        Some(Value::Bytes(bytes)) => Ok(bytes.clone()),
        Some(_) => Err(TorrentError::TorrentParsingError(
            "Peers field is not a byte string".to_string(),
        )
        .into()),
        None => Err(TorrentError::TorrentParsingError("Peers field not found".to_string()).into()),
    }
}

fn parse_peers(peers_compact: Vec<u8>) -> Vec<Peer> {
    peers_compact
        .chunks_exact(6)
        .map(|chunk| {
            let ip = format!("{}.{}.{}.{}", chunk[0], chunk[1], chunk[2], chunk[3]);
            let port = u16::from_be_bytes([chunk[4], chunk[5]]);
            Peer {
                id: uuid::Uuid::new_v4(),
                ip,
                port,
            }
        })
        .collect()
}
