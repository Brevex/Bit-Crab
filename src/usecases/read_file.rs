use std::fs::File;
use std::io::Read;
use crate::domain::errors::TorrentError;

pub fn read_file(file_path: &str) -> Result<Vec<u8>, TorrentError>
{
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
