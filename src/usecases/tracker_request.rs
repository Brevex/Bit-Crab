use crate::domain::entities::{TorrentInfo, TrackerRequest, TrackerResponse};
use crate::domain::errors::TorrentError;
use reqwest;
use std::convert::TryInto;

pub async fn make_tracker_request(torrent_info: &TorrentInfo, peer_id: &str, port: u16) -> Result<TrackerResponse, TorrentError>
{
    let announce_url = get_announce_url(torrent_info)?;
    let info_hash = get_info_hash(torrent_info)?;
    let encoded_info_hash = urlencode(&info_hash);
    let length = get_file_length(torrent_info)?;

    let request = create_tracker_request(peer_id, port, length);
    let tracker_url = build_tracker_url(&announce_url, &request, &encoded_info_hash)?;
    let response_bytes = send_tracker_request(&tracker_url).await?;

    println!("Raw Tracker Response: {:?}", response_bytes);

    parse_tracker_response(&response_bytes)
}

fn get_announce_url(torrent_info: &TorrentInfo) -> Result<String, TorrentError>
{
    torrent_info
        .announce
        .clone()
        .ok_or_else(|| TorrentError::DecodeError("Announce URL not found".to_string()))
}

fn get_info_hash(torrent_info: &TorrentInfo) -> Result<[u8; 20], TorrentError>
{
    let info_hash_hex = torrent_info.info_hash.as_ref()
        .ok_or_else(|| TorrentError::DecodeError("Info hash not found".to_string()))?;
    let info_hash = hex::decode(info_hash_hex)
        .map_err(|_| TorrentError::DecodeError("Failed to decode info hash".to_string()))?;
    info_hash.try_into()
        .map_err(|_| TorrentError::DecodeError("Info hash has incorrect length".to_string()))
}

fn get_file_length(torrent_info: &TorrentInfo) -> Result<usize, TorrentError>
{
    torrent_info.length
        .ok_or_else(|| TorrentError::DecodeError("File length not found".to_string()))
        .map(|len| len as usize)
}

fn create_tracker_request(peer_id: &str, port: u16, length: usize) -> TrackerRequest
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

fn build_tracker_url(announce_url: &str, request: &TrackerRequest, encoded_info_hash: &str) -> Result<String, TorrentError>
{
    let url_params = serde_urlencoded::to_string(request)
        .map_err(|e| TorrentError::DecodeError(format!("url-encode tracker parameters: {}", e)))?;
    Ok(format!("{}?{}&info_hash={}", announce_url, url_params, encoded_info_hash))
}

async fn send_tracker_request(tracker_url: &str) -> Result<bytes::Bytes, TorrentError>
{
    let response = reqwest::get(tracker_url)
        .await
        .map_err(|e| TorrentError::DecodeError(format!("query tracker: {}", e)))?;
    response.bytes()
        .await
        .map_err(|e| TorrentError::DecodeError(format!("fetch tracker response: {}", e)))
}

fn parse_tracker_response(response_bytes: &bytes::Bytes) -> Result<TrackerResponse, TorrentError>
{
    serde_bencode::from_bytes(response_bytes)
        .map_err(|e| TorrentError::DecodeError(format!("parse tracker response: {}", e)))
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
