use thiserror::Error;
use std::io;
use std::num::ParseIntError;
use std::str::Utf8Error;

#[derive(Error, Debug)]
pub enum TorrentError
{
    #[error("IO Error: {0}")]
    IoError(#[from] io::Error),

    #[error("Network Error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("UTF-8 Error: {0}")]
    Utf8Error(#[from] Utf8Error),

    #[error("Parse Int Error: {0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("Torrent parsing error: {0}")]
    TorrentParsingError(String),
}
