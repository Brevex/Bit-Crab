mod domain;
mod usecases;
mod adapters;

use adapters::controllers::torrent_controller::handle_torrent;
use env_logger::Env;
use anyhow::Result;
use log::info;

#[tokio::main]
async fn main() -> Result<()>
{
    env_logger::Builder::from_env(Env::default()
        .default_filter_or("info"))
        .init();

    let file_path = "./src/test.torrent";

    info!("Starting torrent handling for file: {}", file_path);
    handle_torrent(file_path).await?;
    Ok(())
}