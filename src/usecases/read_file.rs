use std::fs::File;
use std::io::Read;
use anyhow::{Context, Result};

pub fn read_file(file_path: &str) -> Result<Vec<u8>>
{
    let mut file = File::open(file_path)
        .context("Failed to open file")?;

    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)
        .context("Failed to read file")?;

    Ok(buffer)
}
