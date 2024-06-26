use serde_json::{self, Value};
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{self, Read};
use std::fmt;

#[derive(Debug)]
enum TorrentError
{
    IoError(io::Error),
    DecodeError(String),
    Utf8Error(std::str::Utf8Error),
    ParseIntError(std::num::ParseIntError),
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

impl From<std::str::Utf8Error> for TorrentError
{
    fn from(error: std::str::Utf8Error) -> Self
    {
        TorrentError::Utf8Error(error)
    }
}

impl From<std::num::ParseIntError> for TorrentError
{
    fn from(error: std::num::ParseIntError) -> Self
    {
        TorrentError::ParseIntError(error)
    }
}

#[derive(Debug)]
struct TorrentInfo
{
    announce: Option<String>,
    length: Option<i64>,
    piece_length: Option<i64>,
    pieces: Option<Vec<String>>,
    info_hash: Option<String>,
}

fn main() -> Result<(), TorrentError>
{
    let file_path = "./src/sample.torrent";
    let buffer = read_file(file_path)?;

    let decoded_value = decode_bencoded_value(&buffer)
        .map_err(|e| TorrentError::DecodeError(format!("Failed to decode bencoded value: {}", e)))?;

    let torrent_info = extract_torrent_info(&decoded_value.0);
    print_torrent_info(&torrent_info);

    Ok(())
}

fn read_file(file_path: &str) -> Result<Vec<u8>, TorrentError>
{
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn extract_torrent_info(decoded_value: &Value) -> TorrentInfo
{
    let announce = decoded_value.get("announce").and_then(|v| v.as_str()).map(String::from);
    let info = decoded_value.get("info").and_then(|v| v.as_object());

    let length = info.and_then(|i| i.get("length")).and_then(|v| v.as_i64());
    let piece_length = info.and_then(|i| i.get("piece length")).and_then(|v| v.as_i64());

    let pieces = info.and_then(|i| i.get("pieces")).and_then(|v| v.as_str()).map(|s| {
        s.as_bytes().chunks(20)
            .map(|chunk| format!("{:x}", Sha1::digest(chunk)))
            .collect()
    });

    let info_hash = info.map(|i| {
        let bencoded_info = bencode_info_dict(i);
        format!("{:x}", Sha1::digest(&bencoded_info))
    });

    TorrentInfo
    {
        announce,
        length,
        piece_length,
        pieces,
        info_hash,
    }
}

fn print_torrent_info(torrent_info: &TorrentInfo)
{
    println!(
        "Tracker URL: {}",
        torrent_info.announce.as_deref().unwrap_or("Tracker URL not found.")
    );

    println!(
        "Length: {}",
        torrent_info
            .length
            .map_or("File length not found.".to_string(), |l| format!("{} bytes", l))
    );

    println!(
        "Piece Length: {}",
        torrent_info
            .piece_length
            .map_or("Piece length not found.".to_string(), |pl| format!("{}", pl))
    );

    if let Some(info_hash) = &torrent_info.info_hash {
        println!("Info Hash: {}", info_hash);
    }

    match &torrent_info.pieces
    {
        Some(pieces) => {
            println!("Piece Hashes:");
            for piece in pieces
            {
                println!("{}", piece);
            }
        }
        None => println!("Pieces not found."),
    }
}


fn decode_bencoded_value(encoded_value: &[u8]) -> Result<(Value, &[u8]), String>
{
    match encoded_value.get(0)
    {
        Some(b'i') => decode_integer(encoded_value),
        Some(b'l') => decode_list(encoded_value),
        Some(b'd') => decode_dict(encoded_value),
        Some(b'0'..=b'9') => decode_string(encoded_value),
        _ => Err(format!("Unhandled encoded value: {:?}", encoded_value)),
    }
}

fn decode_integer(encoded_value: &[u8]) -> Result<(Value, &[u8]), String>
{
    if let Some(end_index) = encoded_value.iter().position(|&b| b == b'e')
    {
        let integer = std::str::from_utf8(&encoded_value[1..end_index])
            .map_err(|_| "Invalid UTF-8".to_string())?
            .parse::<i64>()
            .map_err(|_| "Invalid integer".to_string())?;
        Ok((integer.into(), &encoded_value[end_index + 1..]))
    }
    else { Err("Invalid integer encoding".to_string()) }
}

fn decode_list(encoded_value: &[u8]) -> Result<(Value, &[u8]), String>
{
    let mut values = Vec::new();
    let mut rest = &encoded_value[1..];

    while !rest.is_empty() && rest[0] != b'e'
    {
        let (v, remainder) = decode_bencoded_value(rest)?;
        values.push(v);
        rest = remainder;
    }

    if !rest.is_empty() && rest[0] == b'e' { Ok((values.into(), &rest[1..])) }
    else { Err("Invalid list encoding".to_string()) }
}

fn decode_dict(encoded_value: &[u8]) -> Result<(Value, &[u8]), String>
{
    let mut dict = serde_json::Map::new();
    let mut rest = &encoded_value[1..];

    while !rest.is_empty() && rest[0] != b'e'
    {
        let (k, remainder) = decode_string(rest)?;
        let k = k.as_str().ok_or("Dict keys must be strings".to_string())?.to_string();
        let (v, remainder) = decode_bencoded_value(remainder)?;
        dict.insert(k, v);
        rest = remainder;
    }

    if !rest.is_empty() && rest[0] == b'e' { Ok((dict.into(), &rest[1..])) }
    else { Err("Invalid dictionary encoding".to_string()) }
}

fn decode_string(encoded_value: &[u8]) -> Result<(Value, &[u8]), String>
{
    if let Some(colon_pos) = encoded_value.iter().position(|&b| b == b':')
    {
        if let Ok(len) = std::str::from_utf8(&encoded_value[..colon_pos])
            .map_err(|_| "Invalid UTF-8".to_string())?
            .parse::<usize>()
        {
            let start = colon_pos + 1;
            let end = start + len;
            if encoded_value.len() >= end
            {
                return Ok((
                    String::from_utf8_lossy(&encoded_value[start..end]).into(),
                    &encoded_value[end..],
                ));
            }
        }
    }
    Err("Invalid string encoding".to_string())
}

fn bencode_info_dict(info: &serde_json::Map<String, Value>) -> Vec<u8>
{
    let mut bencoded = Vec::new();
    bencoded.push(b'd');
    let mut keys: Vec<_> = info.keys().collect();
    keys.sort();

    for key in keys
    {
        let value = &info[key];
        bencode_string(&mut bencoded, key);
        bencode_value(&mut bencoded, value);
    }

    bencoded.push(b'e');
    bencoded
}

fn bencode_string(buffer: &mut Vec<u8>, s: &str)
{
    buffer.extend_from_slice(s.len().to_string().as_bytes());
    buffer.push(b':');
    buffer.extend_from_slice(s.as_bytes());
}

fn bencode_value(buffer: &mut Vec<u8>, value: &Value)
{
    match value
    {
        Value::Number(n) if n.is_i64() => {
            buffer.push(b'i');
            buffer.extend_from_slice(n.to_string().as_bytes());
            buffer.push(b'e');
        }
        Value::String(s) => bencode_string(buffer, s),
        Value::Array(arr) => {
            buffer.push(b'l');
            for v in arr
            {
                bencode_value(buffer, v);
            }
            buffer.push(b'e');
        }
        Value::Object(obj) => {
            buffer.push(b'd');
            let mut keys: Vec<_> = obj.keys().collect();
            keys.sort();

            for key in keys
            {
                let v = &obj[key];
                bencode_string(buffer, key);
                bencode_value(buffer, v);
            }
            buffer.push(b'e');
        }
        _ => panic!("Unsupported value type for bencoding"),
    }
}
