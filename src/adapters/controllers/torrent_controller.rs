use log::info;
use sha1::{Digest, Sha1};
use std::net::SocketAddr;
use anyhow::{Context, Result};

use crate::domain::entities::{Peer, Torrent};
use crate::domain::errors::TorrentError;
use crate::adapters::presenters::torrent_presenter::print_torrent_info;
use crate::usecases::{read_file::read_file,
                      peer_discovery::request_peers,
                      peer_handshake::perform_handshake,
                      extract_torrent_info::extract_torrent_info};

pub async fn handle_torrent(file_path: &str) -> Result<()>
{
    info!("Reading torrent file: {}\n", file_path);

    let buffer = read_file(file_path).await?;
    let decoded_value = decode_torrent(&buffer)?;
    let info_hash_hex = calculate_info_hash(&decoded_value)?;
    let torrent_info = extract_torrent_info(&decoded_value, &info_hash_hex);

    print_torrent_info(&torrent_info);
    println!("\n");

    let peers = request_peers(&torrent_info).await?;
    handshake_with_peers(peers, &info_hash_hex).await;

    info!("Torrent handling completed successfully.\n");
    Ok(())
}

fn decode_torrent(buffer: &[u8]) -> Result<Torrent, TorrentError>
{
    serde_bencode::from_bytes(buffer)
        .map_err(|e| TorrentError::TorrentParsingError(e.to_string()))
}

fn calculate_info_hash(decoded_value: &Torrent) -> Result<String>
{
    let mut hasher = Sha1::new();
    let info_encoded = serde_bencode::to_bytes(&decoded_value.info)
        .context("Failed to re-encode info section")?;

    hasher.update(&info_encoded);
    Ok(hex::encode(hasher.finalize()))
}

async fn handshake_with_peers(peers: Vec<Peer>, info_hash_hex: &str)
{
    for peer in peers
    {
        let peer_addr: SocketAddr = format!("{}:{}", peer.ip, peer.port)
            .parse()
            .context("Invalid peer address format")
            .unwrap();

        match perform_handshake(&peer_addr, &hex::decode(info_hash_hex)
            .unwrap()).await
        {
            Ok(peer_id) => info!("Handshake successful with peer ID: {}\n",
                hex::encode(peer_id)),

            Err(e) => info!("Failed to handshake with peer {}: {}\n",
                peer_addr, e),
        }
    }
}
