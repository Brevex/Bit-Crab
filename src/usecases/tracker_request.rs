use crate::domain::entities::{TorrentInfo, TrackerRequest, TrackerResponse};
use anyhow::{Context, Result};
use std::convert::TryInto;

pub async fn make_tracker_request(
    torrent_info: &TorrentInfo,
    peer_id: &str,
    port: u16) -> Result<TrackerResponse>
{
    let announce_url = get_announce_url(torrent_info)?;
    let info_hash = get_info_hash(torrent_info)?;
    let encoded_info_hash = urlencode(&info_hash);
    let length = get_file_length(torrent_info)?;

    let request = create_tracker_request(peer_id, port, length);
    let tracker_url = build_tracker_url(
        &announce_url,
        &request,
        &encoded_info_hash)?;
    let response_bytes = send_tracker_request(&tracker_url).await?;

    println!("Raw Tracker Response: {:?}", response_bytes);

    parse_tracker_response(&response_bytes)
}

fn get_announce_url(torrent_info: &TorrentInfo) -> Result<String>
{
    torrent_info
        .announce
        .clone()
        .ok_or_else(|| anyhow::anyhow!("Announce URL not found"))
}

fn get_info_hash(torrent_info: &TorrentInfo) -> Result<[u8; 20]>
{
    let info_hash_hex = torrent_info.info_hash.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Info hash not found"))?;
    let info_hash = hex::decode(info_hash_hex)
        .context("Failed to decode info hash")?;

    info_hash.try_into()
        .map_err(|_| anyhow::anyhow!("Info hash has incorrect length"))
}

fn get_file_length(torrent_info: &TorrentInfo) -> Result<usize>
{
    torrent_info.length
        .ok_or_else(|| anyhow::anyhow!("File length not found"))
        .map(|len| len as usize)
}

fn create_tracker_request(
    peer_id: &str,
    port: u16,
    length: usize) -> TrackerRequest
{
    TrackerRequest
    {
        peer_id: peer_id.to_string(),
        port,
        uploaded: 0,
        downloaded: 0,
        left: length,
        compact: 1,
    }
}

fn build_tracker_url(
    announce_url: &str,
    request: &TrackerRequest,
    encoded_info_hash: &str) -> Result<String>
{
    let url_params = serde_urlencoded::to_string(request)
        .context("url-encode tracker parameters")?;

    Ok(format!("{}?{}&info_hash={}",
               announce_url,
               url_params,
               encoded_info_hash))
}

async fn send_tracker_request(tracker_url: &str) -> Result<bytes::Bytes>
{
    let response = reqwest::get(tracker_url)
        .await
        .context("query tracker")?;

    response.bytes()
        .await
        .context("fetch tracker response")
}

fn parse_tracker_response(response_bytes: &bytes::Bytes) -> Result<TrackerResponse>
{
    serde_bencode::from_bytes(response_bytes)
        .context("parse tracker response")
}

fn urlencode(t: &[u8; 20]) -> String
{
    let mut encoded = String::with_capacity(3 * t.len());

    for &byte in t
    {
        encoded.push('%');
        encoded.push_str(&format!("{:02X}", byte));
    }
    encoded
}
