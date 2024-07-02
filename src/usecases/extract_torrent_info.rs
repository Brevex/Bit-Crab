use crate::domain::entities::info::{Keys, TorrentInfo};
use crate::domain::entities::torrent::Torrent;

pub fn extract_torrent_info(decoded_value: &Torrent, info_hash: &[u8; 20]) -> TorrentInfo
{
    let announce = Some(decoded_value.announce.clone());
    let length = if let Keys::SingleFile { length } = &decoded_value.info.keys
    {
        Some(*length as i64)
    }
    else { None };

    let piece_length = Some(decoded_value.info.piece_length as i64);
    let pieces = Some(decoded_value.info.pieces.clone());
    let info = Some(decoded_value.info.clone());

    TorrentInfo
    {
        announce,
        length,
        piece_length,
        pieces,
        info,
        info_hash: Some(*info_hash),
    }
}
