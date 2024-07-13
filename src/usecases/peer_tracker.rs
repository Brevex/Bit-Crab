use crate::entities::peer::{Peer, TrackerRequest, TrackerResponse};
use crate::entities::torrent::Torrent;
use crate::utils::errors::{MetadataError, TorrentError};
use crate::utils::extract_torrent_metadata::extract_int;

use anyhow::Result;
use serde_bencode::value::Value;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::str;
use tokio::net::UdpSocket;
use tokio::time::{timeout, Duration};
use url::Url;

pub async fn discover_peers(torrent: &Torrent) -> Result<TrackerResponse, TorrentError>
{
    let tracker_request = TrackerRequest::new(torrent);
    let url = tracker_request.build_url();

    if url.starts_with("udp://")
    {
        discover_peers_udp(&tracker_request).await
    }
    else
    {
        discover_peers_http(&tracker_request).await
    }
}

async fn discover_peers_http(
    tracker_request: &TrackerRequest,
) -> Result<TrackerResponse, TorrentError>
{
    let url = tracker_request.build_url();
    let response = reqwest::get(&url).await?.bytes().await?;
    let value: Value = serde_bencode::from_bytes(&response)?;

    if let Value::Dict(dict) = value
    {
        if let Some(failure_reason) = dict.get(&b"failure reason"[..])
        {
            let reason = decode_failure_reason(failure_reason)?;
            return Err(MetadataError::FieldError(reason).into());
        }
        log_tracker_response(&dict);
        let interval = extract_int("interval", &dict)?;
        let peers = extract_peers("peers", &dict)?;

        Ok(TrackerResponse::new(interval, peers))
    }
    else { Err(MetadataError::IncorrectFormatError.into()) }
}

async fn discover_peers_udp(
    tracker_request: &TrackerRequest,
) -> Result<TrackerResponse, TorrentError>
{
    let url = Url::parse(tracker_request.tracker_url().as_str())?;
    let addr = format!(
        "{}:{}",
        url.host_str()
            .ok_or(MetadataError::InvalidUrl(url.to_string()))?,
        url.port_or_known_default()
            .ok_or(MetadataError::InvalidUrl(url.to_string()))?
    );

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let mut buf = [0; 4096];

    let connection_id: u64 = 0x41727101980;
    let transaction_id: u32 = rand::random::<u32>();

    let mut request = Vec::new();
    request.extend(&connection_id.to_be_bytes());
    request.extend(&0_u32.to_be_bytes());
    request.extend(&transaction_id.to_be_bytes());

    socket.send_to(&request, &addr).await?;

    let n = timeout(Duration::from_secs(2), socket.recv(&mut buf)).await??;

    if n < 16
    {
        return Err(MetadataError::IncorrectFormatError.into());
    }

    let action = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let response_transaction_id = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);

    if action != 0 || response_transaction_id != transaction_id
    {
        return Err(MetadataError::IncorrectFormatError.into());
    }

    let connection_id = u64::from_be_bytes([
        buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
    ]);

    let mut request = Vec::new();
    request.extend(&connection_id.to_be_bytes());
    request.extend(&1_u32.to_be_bytes());
    request.extend(&transaction_id.to_be_bytes());
    request.extend(tracker_request.info_hash().iter());
    request.extend(tracker_request.peer_id().as_bytes());
    request.extend(&0_u64.to_be_bytes());
    request.extend(&tracker_request.left().to_be_bytes());
    request.extend(&0_u64.to_be_bytes());
    request.extend(&2_u32.to_be_bytes());
    request.extend(&0_u32.to_be_bytes());
    request.extend(&0_u32.to_be_bytes());
    request.extend(&u32::MAX.to_be_bytes());
    request.extend(&tracker_request.port().to_be_bytes());

    socket.send_to(&request, &addr).await?;

    let n = timeout(Duration::from_secs(2), socket.recv(&mut buf)).await??;

    if n < 20
    {
        return Err(MetadataError::IncorrectFormatError.into());
    }

    let action = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let response_transaction_id = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);

    if action != 1 || response_transaction_id != transaction_id
    {
        return Err(MetadataError::IncorrectFormatError.into());
    }

    let interval = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]) as i64;
    let mut peers = Vec::new();
    let mut offset = 20;

    while offset < n
    {
        let ip = IpAddr::V4(Ipv4Addr::new(
            buf[offset],
            buf[offset + 1],
            buf[offset + 2],
            buf[offset + 3],
        ));
        let port = u16::from_be_bytes([buf[offset + 4], buf[offset + 5]]);
        peers.push(Peer::new(ip, port));
        offset += 6;
    }
    Ok(TrackerResponse::new(interval, peers))
}

fn extract_peers(key: &str, dict: &HashMap<Vec<u8>, Value>) -> Result<Vec<Peer>, MetadataError>
{
    let peers_bytes = dict
        .get(key.as_bytes())
        .and_then(|v| match v {
            Value::Bytes(b) => Some(b),
            _ => None,
        })
        .ok_or(MetadataError::FieldError(key.to_string()))?;

    let peers = peers_bytes
        .chunks(6)
        .map(|chunk| {
            let ip = IpAddr::V4(Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]));
            let port = u16::from_be_bytes([chunk[4], chunk[5]]);
            Peer::new(ip, port)
        })
        .collect();

    Ok(peers)
}

fn log_tracker_response(dict: &HashMap<Vec<u8>, Value>)
{
    for (key, value) in dict
    {
        if let Ok(key_str) = String::from_utf8(key.clone())
        {
            println!("Key: {}, Value: {:?}", key_str, value);
        }
    }
}

fn decode_failure_reason(value: &Value) -> Result<String, MetadataError>
{
    if let Value::Bytes(bytes) = value
    {
        String::from_utf8(bytes.clone()).map_err(|_| MetadataError::IncorrectFormatError)
    }
    else { Err(MetadataError::IncorrectFormatError) }
}
