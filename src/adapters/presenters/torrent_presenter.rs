use crate::domain::entities::info::TorrentInfo;

pub fn print_torrent_info(torrent_info: &TorrentInfo) {
    println!("Tracker URL: {}", torrent_info.announce);

    println!("Length: {} bytes", torrent_info.length);

    println!("Piece Length: {}", torrent_info.piece_length);

    println!("Info Hash: {:?}", torrent_info.info_hash);
}

pub fn print_torrent_pieces(torrent_info: &TorrentInfo) {
    println!("Piece Hashes:");

    if let Ok(pieces) = torrent_info.pieces.to_hash_vec() {
        for piece in pieces {
            println!("{:x?}", piece);
        }
    } else {
        eprintln!("Failed to convert pieces to hash vector");
    }
}
