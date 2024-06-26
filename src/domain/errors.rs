use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::str::Utf8Error;

#[derive(Debug)]
pub enum TorrentError
{
    IoError(io::Error),
    DecodeError(String),
    Utf8Error(Utf8Error),
    ParseIntError(ParseIntError),
}

impl fmt::Display for TorrentError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            TorrentError::IoError(e) => write!(f, "IO Error: {}", e),
            TorrentError::DecodeError(e) => write!(f, "Decode Error: {}", e),
            TorrentError::Utf8Error(e) => write!(f, "UTF-8 Error: {}", e),
            TorrentError::ParseIntError(e) => write!(f, "Parse Int Error: {}", e),
        }
    }
}

impl From<io::Error> for TorrentError
{
    fn from(error: io::Error) -> Self
    {
        TorrentError::IoError(error)
    }
}

impl From<Utf8Error> for TorrentError
{
    fn from(error: Utf8Error) -> Self
    {
        TorrentError::Utf8Error(error)
    }
}

impl From<ParseIntError> for TorrentError
{
    fn from(error: ParseIntError) -> Self
    {
        TorrentError::ParseIntError(error)
    }
}
