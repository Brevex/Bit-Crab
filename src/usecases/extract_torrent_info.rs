use crate::domain::entities::info::{Keys, TorrentInfo};
use crate::domain::entities::torrent::Torrent;

pub fn extract_torrent_info(decoded_value: &Torrent, info_hash: &[u8; 20]) -> TorrentInfo {
    let announce = decoded_value.announce.clone();
    let length = if let Keys::SingleFile { length } = &decoded_value.info.keys {
        *length as i64
    } else {
        0
    };

    let piece_length = decoded_value.info.piece_length as i64;
    let pieces = decoded_value.info.pieces.clone();
    let info = decoded_value.info.clone();

    TorrentInfo {
        announce,
        length,
        piece_length,
        pieces,
        info,
        info_hash: *info_hash,
    }
}
