use serde_json::Value;
use sha1::{Digest, Sha1};

use crate::domain::entities::TorrentInfo;

pub fn extract_torrent_info(decoded_value: &Value) -> TorrentInfo
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
