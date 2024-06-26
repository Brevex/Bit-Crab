#[derive(Debug)]
pub struct TorrentInfo
{
    pub announce: Option<String>,
    pub length: Option<i64>,
    pub piece_length: Option<i64>,
    pub pieces: Option<Vec<String>>,
    pub info_hash: Option<String>,
}
