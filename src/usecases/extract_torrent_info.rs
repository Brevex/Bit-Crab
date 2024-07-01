use crate::domain::entities::{Keys, Torrent, TorrentInfo};

pub fn extract_torrent_info(decoded_value: &Torrent, info_hash: &str) -> TorrentInfo {
    let announce = Some(decoded_value.announce.clone());
    let length = if let Keys::SingleFile { length } = &decoded_value.info.keys {
        Some(*length as i64)
    } else { None };

    let piece_length = Some(decoded_value.info.piece_length as i64);
    let pieces = Some(decoded_value.info
        .pieces.to_hash_vec()
        .iter().map(hex::encode)
        .collect()
    );

    TorrentInfo {
        announce,
        length,
        piece_length,
        pieces,
        info_hash: Some(info_hash.to_string()),
    }
}
