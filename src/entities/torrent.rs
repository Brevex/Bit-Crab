use getset::Getters;
use reqwest::Url;
use serde::Deserialize;

#[derive(Getters, Clone, Debug)]
pub struct Torrent
{
    #[get = "pub"]
    announce: Url,
    #[get = "pub"]
    info: TorrentInfo,
    #[get = "pub"]
    info_hash: [u8; 20],
}

impl Torrent
{
    pub fn new(announce: Url, info: TorrentInfo, info_hash: [u8; 20]) -> Self
    {
        Self
        {
            announce,
            info,
            info_hash,
        }
    }
}

#[derive(Getters, Clone, Debug)]
pub struct TorrentInfo
{
    #[get = "pub"]
    name: String,
    #[get = "pub"]
    piece_length: i64,
    #[get = "pub"]
    pieces: Vec<u8>,
    #[get = "pub"]
    length: i64,
    #[get = "pub"]
    files: Vec<FileInfo>,
}

impl TorrentInfo
{
    pub fn new(
        name: String,
        piece_length: i64,
        pieces: Vec<u8>,
        length: i64,
        files: Vec<FileInfo>,
    ) -> Self
    {
        Self
        {
            name,
            piece_length,
            pieces,
            length,
            files,
        }
    }

    pub fn piece_hashes(&self) -> Vec<[u8; 20]>
    {
        self.pieces
            .chunks(20)
            .map(|chunk| {
                let mut hash = [0u8; 20];
                hash.copy_from_slice(chunk);
                hash
            })
            .collect()
    }
}

#[derive(Getters, Deserialize, Clone, Debug)]
pub struct FileInfo
{
    #[get = "pub"]
    length: i64,
    #[get = "pub"]
    path: Vec<String>,
}

impl FileInfo
{
    pub fn new(length: i64, path: Vec<String>) -> Self
    {
        Self { length, path }
    }
}
