use anyhow::{Context, Result};
use sha1::{Digest, Sha1};
use crate::usecases::extract_torrent_info::extract_torrent_info;
use crate::usecases::read_file::read_file;
use crate::usecases::tracker_request::make_tracker_request;
use crate::adapters::presenters::torrent_presenter::print_torrent_info;
use crate::domain::entities::Torrent;

pub async fn handle_torrent(file_path: &str) -> Result<()>
{
    let buffer = read_file(file_path)?;

    let decoded_value: Torrent = serde_bencode::from_bytes(&buffer)
        .context("parse torrent file")?;

    let info_encoded = serde_bencode::to_bytes(&decoded_value.info)
        .context("re-encode info section")?;

    let mut hasher = Sha1::new();
    hasher.update(&info_encoded);
    let info_hash = hasher.finalize();
    let info_hash_hex = hex::encode(&info_hash);

    let torrent_info = extract_torrent_info(
        &decoded_value,
        &info_hash_hex);

    print_torrent_info(&torrent_info);

    let peer_id = "00112233445566778899";
    let port = 6881;
    let tracker_response = make_tracker_request(
        &torrent_info,
        peer_id,
        port).await?;

    if let Some(peers) = tracker_response.peers
    {
        for peer in peers.0
        {
            println!("{}:{}", peer.ip(), peer.port());
        }
    }
    else { println!("No peers found in the tracker response."); }
    Ok(())
}
