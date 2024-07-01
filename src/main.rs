use anyhow::Result;
use env_logger::Env;
use log::info;

use adapters::controllers::torrent_controller::handle_torrent;

mod adapters;
mod domain;
mod usecases;

#[tokio::main]
async fn main() -> Result<()>
{
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let file_path = "./src/test.torrent";

    info!("Starting torrent handling for file: {}", file_path);
    handle_torrent(file_path).await?;
    Ok(())
}
