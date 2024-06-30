use anyhow::{Context, Result};
use serde_bencode::value::Value;
use reqwest::Client;
use rand::{distributions::Alphanumeric, Rng};
use std::collections::HashMap;
use crate::domain::entities::TorrentInfo;
use crate::domain::errors::TorrentError;

pub async fn request_peers(torrent_info: &TorrentInfo) -> Result<Vec<(String, u16)>>
{
    let client = Client::new();

    let tracker_url = torrent_info
        .announce
        .as_ref()
        .ok_or_else(|| TorrentError::TorrentParsingError(
            "Tracker URL not found".to_string())
        )?;

    let info_hash = hex::decode(torrent_info
        .info_hash
        .as_ref()
        .ok_or_else(|| TorrentError::TorrentParsingError(
            "Info hash not found".to_string())
        )?)
        .context("Failed to decode info hash")?;

    let info_hash_encoded: String = info_hash
        .iter()
        .map(|byte| format!("%{:02x}", byte))
        .collect();

    let peer_id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();

    let length = torrent_info
        .length
        .ok_or_else(|| TorrentError::TorrentParsingError(
            "File length not found".to_string())
        )?;

    let url = format!(
        "{}?info_hash={}&peer_id={}&port=6881&uploaded=0&downloaded=0&left={}&compact=1",
        tracker_url,
        info_hash_encoded,
        peer_id,
        length as u64
    );

    let response = client
        .get(&url)
        .send()
        .await?
        .bytes()
        .await
        .context("Failed to read response from tracker")?;

    let decoded_response: HashMap<String, Value> = serde_bencode::from_bytes(&response)
        .context("Failed to decode bencode response")?;

    let peers_compact = decoded_response
        .get("peers")
        .and_then(|v| match v
        {
            Value::Bytes(bytes) => Some(bytes.as_slice()),
            _ => None,
        })
        .ok_or_else(|| TorrentError::TorrentParsingError(
            "Peers not found".to_string())
        )?;

    Ok(peers_compact.chunks_exact(6).map(|chunk|
    {
        let ip = format!("{}.{}.{}.{}",
                         chunk[0], chunk[1],
                         chunk[2], chunk[3]);

        let port = u16::from_be_bytes([chunk[4], chunk[5]]);
        (ip, port)
    }).collect())
}