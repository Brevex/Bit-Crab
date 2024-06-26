use crate::domain::entities::TorrentInfo;

pub fn print_torrent_info(torrent_info: &TorrentInfo)
{
    println!(
        "Tracker URL: {}",
        torrent_info.announce.as_deref().unwrap_or("Tracker URL not found.")
    );

    println!(
        "Length: {}",
        torrent_info
            .length
            .map_or("File length not found.".to_string(), |l| format!("{} bytes", l))
    );

    println!(
        "Piece Length: {}",
        torrent_info
            .piece_length
            .map_or("Piece length not found.".to_string(), |pl| format!("{}", pl))
    );

    if let Some(info_hash) = &torrent_info.info_hash
    {
        println!("Info Hash: {}", info_hash);
    }

    match &torrent_info.pieces
    {
        Some(pieces) => {
            println!("Piece Hashes:");
            for piece in pieces
            {
                println!("{}", piece);
            }
        }
        None => println!("Pieces not found."),
    }
}
