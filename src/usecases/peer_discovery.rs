use url::Url;
use rand::Rng;
use uuid::Uuid;
use reqwest::Client;
use anyhow::{Context, Result};
use std::collections::HashMap;
use serde_bencode::value::Value;
use rand::distributions::Alphanumeric;
use crate::domain::errors::TorrentError;
use crate::domain::entities::{TorrentInfo, Peer};

pub async fn request_peers(torrent_info: &TorrentInfo) -> Result<Vec<Peer>>
{
    let tracker_url = get_tracker_url(torrent_info)?;
    let info_hash_encoded = get_info_hash_encoded(torrent_info)?;
    let peer_id = generate_peer_id();
    let length = get_file_length(torrent_info)?;

    let url = format!(
        "{}?info_hash={}&peer_id={}&port=6881&uploaded=0&downloaded=0&left={}&compact=1",
        tracker_url, info_hash_encoded, peer_id, length as u64
    );

    let response = send_tracker_request(&url).await?;
    let peers_compact = extract_peers_compact(&response)?;
    let peers = parse_peers(peers_compact);

    Ok(peers)
}

fn get_tracker_url(torrent_info: &TorrentInfo) -> std::result::Result<&Url, TorrentError>
{
    torrent_info
        .announce
        .as_ref()
        .ok_or_else(|| TorrentError::TorrentParsingError(
            "Tracker URL not found".to_string()
        ))
}

fn get_info_hash_encoded(torrent_info: &TorrentInfo) -> Result<String>
{
    let info_hash = hex::decode(
        torrent_info
            .info_hash
            .as_ref()
            .ok_or_else(|| TorrentError::TorrentParsingError(
                "Info hash not found".to_string()
            ))?,
    )
        .context("Failed to decode info hash")?;

    Ok(info_hash
        .iter()
        .map(|byte| format!("%{:02x}", byte))
        .collect::<String>())
}

fn generate_peer_id() -> String
{
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect()
}

fn get_file_length(torrent_info: &TorrentInfo) -> std::result::Result<i64, TorrentError>
{
    torrent_info
        .length
        .ok_or_else(|| TorrentError::TorrentParsingError(
            "File length not found".to_string()
        ))
}

async fn send_tracker_request(url: &str) -> Result<bytes::Bytes>
{
    let client = Client::new();
    let response = client.get(url).send().await?.bytes().await?;
    Ok(response)
}

fn extract_peers_compact(response: &bytes::Bytes) -> Result<Vec<u8>>
{
    let decoded_response: HashMap<String, Value> =
        serde_bencode::from_bytes(&response)
            .context("Failed to decode bencode response")?;

    match decoded_response.get("peers")
    {
        Some(Value::Bytes(bytes)) => Ok(bytes.clone()),
        _ => Err(TorrentError::TorrentParsingError(
            "Peers not found".to_string()).into()
        ),
    }
}

fn parse_peers(peers_compact: Vec<u8>) -> Vec<Peer>
{
    peers_compact
        .chunks_exact(6)
        .map(|chunk| {
            let ip = format!("{}.{}.{}.{}", chunk[0], chunk[1], chunk[2], chunk[3]);
            let port = u16::from_be_bytes([chunk[4], chunk[5]]);
            Peer
            {
                id: Uuid::new_v4(),
                ip,
                port,
            }
        })
        .collect()
}
