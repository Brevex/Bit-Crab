mod domain;
mod usecases;
mod adapters;

use adapters::controllers::torrent_controller::handle_torrent;

#[tokio::main]
async fn main() -> Result<(), domain::errors::TorrentError>
{
    let file_path = "./src/test.torrent";
    handle_torrent(file_path).await
}
