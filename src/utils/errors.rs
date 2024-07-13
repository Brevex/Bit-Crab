use reqwest::Error as ReqwestError;
use serde_bencode::Error as BencodeError;
use std::io::Error as IoError;
use std::net::AddrParseError;
use std::path::PathBuf;
use thiserror::Error;
use tokio::time::error::Elapsed;
use url::ParseError;

#[derive(Debug, Error)]
pub enum FileError
{
    #[error("Failed to read file metadata: {0}")]
    MetadataReadError(PathBuf),

    #[error("File size exceeds 50kb")]
    FileSizeError,

    #[error("Failed to read file: {0}")]
    FileReadError(PathBuf),

    #[error(transparent)]
    IoError(#[from] IoError),
}

#[derive(Debug, Error)]
pub enum MetadataError
{
    #[error("Failed to deserialize file content")]
    DeserializationError,

    #[error("Incorrect format, dictionary expected")]
    IncorrectFormatError,

    #[error("Missing or invalid field: {0}")]
    FieldError(String),

    #[error("Invalid announce URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid value: {0} must be positive")]
    InvalidPositiveValue(i64),

    #[error("'length' field must be positive")]
    InvalidLength,

    #[error("'piece length' field must be positive")]
    InvalidPieceLength,

    #[error("'pieces' field cannot be empty")]
    EmptyPiecesField,

    #[error(transparent)]
    BencodeError(#[from] BencodeError),

    #[error(transparent)]
    ParseError(#[from] ParseError),
}

#[derive(Debug, Error)]
pub enum HandshakeError
{
    #[error("Failed to connect to peer at {0}")]
    ConnectionError(String),

    #[error("Failed to send handshake to peer at {0}")]
    HandshakeSendError(String),

    #[error("Failed to receive handshake from peer at {0}")]
    HandshakeReceiveError(String),

    #[error("Invalid handshake response from peer at {0}")]
    InvalidHandshakeResponse(String),

    #[error("Handshake timed out with peer at {0}")]
    HandshakeTimeout(String),

    #[error(transparent)]
    AddrParseError(#[from] AddrParseError),

    #[error(transparent)]
    Elapsed(#[from] Elapsed),
}

#[derive(Debug, Error)]
pub enum TorrentError
{
    #[error(transparent)]
    FileError(#[from] FileError),

    #[error(transparent)]
    MetadataError(#[from] MetadataError),

    #[error(transparent)]
    HandshakeError(#[from] HandshakeError),

    #[error(transparent)]
    ReqwestError(#[from] ReqwestError),

    #[error(transparent)]
    IoError(#[from] IoError),

    #[error(transparent)]
    ParseError(#[from] ParseError),

    #[error(transparent)]
    BencodeError(#[from] BencodeError),

    #[error(transparent)]
    Elapsed(#[from] Elapsed),
}
