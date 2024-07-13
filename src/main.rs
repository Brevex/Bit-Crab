mod entities;
pub mod usecases;
pub mod utils;

use crate::usecases::parse_torrent_file::{
    parse_torrent_file, print_torrent_info, process_torrent,
};
use std::path::PathBuf;

#[tokio::main]
async fn main()
{
    let file_path = "./src/test3.torrent";

    match parse_torrent_file(PathBuf::from(file_path)).await
    {
        Ok(torrent) => {
            print_torrent_info(&torrent).await;
            process_torrent(&torrent).await;
        }
        Err(e) => { eprintln!("Failed to parse torrent file: {}", e); }
    }
}
