use crate::domain::entities::TorrentInfo;

pub fn print_torrent_info(torrent_info: &TorrentInfo)
{
    print_tracker_url(torrent_info);
    print_length(torrent_info);
    print_piece_length(torrent_info);
    print_info_hash(torrent_info);
    print_pieces(torrent_info);
}

fn print_tracker_url(torrent_info: &TorrentInfo)
{
    let tracker_url = torrent_info
        .announce
        .as_deref()
        .unwrap_or("Tracker URL not found.");

    println!("Tracker URL: {}", tracker_url);
}

fn print_length(torrent_info: &TorrentInfo)
{
    let length = torrent_info
        .length
        .map_or("File length not found.".to_string(), |l| format!("{} bytes", l));

    println!("Length: {}", length);
}

fn print_piece_length(torrent_info: &TorrentInfo)
{
    let piece_length = torrent_info
        .piece_length
        .map_or("Piece length not found.".to_string(), |pl| format!("{}", pl));

    println!("Piece Length: {}", piece_length);
}

fn print_info_hash(torrent_info: &TorrentInfo)
{
    if let Some(info_hash) = &torrent_info.info_hash
    {
        println!("Info Hash: {}", info_hash);
    }
}

fn print_pieces(torrent_info: &TorrentInfo)
{
    match &torrent_info.pieces
    {
        Some(pieces) => {
            println!("Piece Hashes:");
            for piece in pieces { println!("{}", piece); }
        }
        None => println!("Pieces not found."),
    }
}
