use tokio::fs::File;
use tokio::io::AsyncReadExt;
use anyhow::Result;

pub async fn read_file(file_path: &str) -> Result<Vec<u8>>
{
    let mut file = File::open(file_path).await?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer).await?;
    Ok(buffer)
}
