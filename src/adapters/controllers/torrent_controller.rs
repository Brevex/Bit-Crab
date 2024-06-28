use log::info;
use anyhow::{Context, Result};
use sha1::{Digest, Sha1};
use crate::usecases::extract_torrent_info::extract_torrent_info;
use crate::usecases::read_file::read_file;
use crate::adapters::presenters::torrent_presenter::print_torrent_info;
use crate::domain::entities::Torrent;
use crate::domain::errors::TorrentError;

pub async fn handle_torrent(file_path: &str) -> Result<()>
{
    info!("Reading torrent file: {}", file_path);

    let buffer = read_file(file_path).await?;
    let decoded_value = decode_torrent(&buffer)?;
    let info_hash_hex = calculate_info_hash(&decoded_value)?;
    let torrent_info = extract_torrent_info(&decoded_value, &info_hash_hex);

    print_torrent_info(&torrent_info);
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
