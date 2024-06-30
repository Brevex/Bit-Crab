use log::info;
use anyhow::{Context, Result};
use sha1::{Digest, Sha1};
use std::net::SocketAddr;
use crate::usecases::extract_torrent_info::extract_torrent_info;
use crate::usecases::read_file::read_file;
use crate::adapters::presenters::torrent_presenter::print_torrent_info;
use crate::domain::entities::Torrent;
use crate::domain::errors::TorrentError;
use crate::usecases::peer_discovery::request_peers;
use crate::usecases::peer_handshake::perform_handshake;

pub async fn handle_torrent(file_path: &str) -> Result<()>
{
    info!("\nReading torrent file: {}", file_path);

    let buffer = read_file(file_path).await?;
    let decoded_value = decode_torrent(&buffer)?;
    let info_hash_hex = calculate_info_hash(&decoded_value)?;
    let torrent_info = extract_torrent_info(&decoded_value, &info_hash_hex);

    print_torrent_info(&torrent_info);

    let peers = request_peers(&torrent_info).await?;

    for (ip, port) in peers
    {
        println!("Peer: {}:{}", ip, port);
        let mut peer_addr: SocketAddr = format!(
            "{}:{}", ip, port)
            .parse()
            .unwrap();

        match perform_handshake(
            &mut peer_addr,
            &hex::decode(info_hash_hex.clone())?).await
        {
            Ok(peer_id) => println!(
                "Handshake successful with peer ID: {}",
                hex::encode(peer_id)),

            Err(e) => eprintln!(
                "Failed to handshake with peer {}: {}",
                peer_addr, e),
        }
    }

    info!("Torrent handling completed successfully.");
    Ok(())
}

fn decode_torrent(buffer: &[u8]) -> Result<Torrent, TorrentError>
{
    serde_bencode::from_bytes(&buffer)
        .map_err(|e| TorrentError::TorrentParsingError(e.to_string()))
}

fn calculate_info_hash(decoded_value: &Torrent) -> Result<String>
{
    let info_encoded = serde_bencode::to_bytes(&decoded_value.info)
        .context("Failed to re-encode info section")?;

    let mut hasher = Sha1::new();
    hasher.update(&info_encoded);
    let info_hash = hasher.finalize();
    Ok(hex::encode(&info_hash))
}
