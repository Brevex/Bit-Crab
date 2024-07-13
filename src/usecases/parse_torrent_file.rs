use crate::entities::torrent::{Torrent, TorrentInfo};
use crate::usecases::download_torrent::download_torrent;
use crate::usecases::peer_tracker::discover_peers;
use crate::usecases::perform_handshake::perform_handshake;
use crate::utils::errors::{FileError, MetadataError, TorrentError};
use crate::utils::extract_torrent_metadata::{
    extract_bytes, extract_dict, extract_files, extract_int, extract_string,
};

use anyhow::Result;
use reqwest::Url;
use serde_bencode::value::Value;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::path::PathBuf;

pub async fn parse_torrent_file<T: Into<PathBuf>>(file_path: T) -> Result<Torrent, TorrentError>
{
    let content = std::fs::read(file_path.into()).map_err(FileError::IoError)?;
    let value: Value = serde_bencode::from_bytes(&content).map_err(MetadataError::BencodeError)?;

    if let Value::Dict(d) = value
    {
        let announce = extract_string("announce", &d)?;
        let info = extract_dict("info", &d)?;
        let info_hash = calculate_info_hash(&info)?;
        let length = extract_int("length", &info).unwrap_or(0);
        let files = extract_files(&info).unwrap_or_default();

        Ok(Torrent::new(
            Url::parse(&announce)?,
            TorrentInfo::new(
                extract_string("name", &info)?,
                extract_int("piece length", &info)?,
                extract_bytes("pieces", &info)?,
                length,
                files,
            ),
            info_hash,
        ))
    }
    else
    {
        Err(TorrentError::MetadataError(
            MetadataError::IncorrectFormatError,
        ))
    }
}

pub async fn print_torrent_info(torrent: &Torrent)
{
    println!("Tracker URL: {}", torrent.announce());
    println!("Name: {}", torrent.info().name());
    println!("Piece Length: {}", torrent.info().piece_length());
    println!("Total Length: {}", torrent.info().length());
    println!("Info Hash: {}", hex::encode(torrent.info_hash()));
    println!("Piece Hashes:");

    for hash in torrent.info().piece_hashes()
    {
        println!("{}", hex::encode(hash));
    }
}

pub async fn process_torrent(torrent: &Torrent)
{
    match discover_peers(torrent).await
    {
        Ok(tracker_response) => {
            println!("Interval: {}", tracker_response.interval());
            println!("Peers:");

            for peer in tracker_response.peers()
            {
                println!("{}:{}", peer.ip(), peer.port());
            }

            match perform_handshake(torrent, tracker_response.peers()).await
            {
                Ok(connected_peers) => {
                    println!("Connected Peers:");

                    for peer in &connected_peers
                    {
                        println!("{}:{}", peer.ip(), peer.port());
                    }
                    if let Err(e) = download_torrent(torrent, &connected_peers).await
                    {
                        eprintln!("Failed to download torrent: {}", e);
                    }
                }
                Err(e) => {
                    println!("Handshake process encountered an error: {}", e);
                }
            }
        }
        Err(e) => { println!("Failed to discover peers: {}", e); }
    }
}

fn calculate_info_hash(info_dict: &HashMap<Vec<u8>, Value>) -> Result<[u8; 20], MetadataError>
{
    let info_bencode = serde_bencode::to_bytes(&Value::Dict(info_dict.clone()))?;

    let mut hasher = Sha1::new();
    hasher.update(&info_bencode);
    let info_hash = hasher.finalize();
    Ok(info_hash.into())
}
