use crate::domain::entities::{Keys, Torrent, TorrentInfo};

pub fn extract_torrent_info(decoded_value: &Torrent, info_hash: &str) -> TorrentInfo
{
    let announce = Some(decoded_value.announce.clone());
    let length = match &decoded_value.info.keys
    {
        Keys::SingleFile { length } => Some(*length as i64),
        _ => None,
    };
    let piece_length = Some(decoded_value.info.piece_length as i64);
    let pieces = Some(decoded_value.info.pieces.0.iter().map(|hash| hex::encode(hash)).collect());

    TorrentInfo
    {
        announce,
        length,
        piece_length,
        pieces,
        info_hash: Some(info_hash.to_string()),
    }
}
