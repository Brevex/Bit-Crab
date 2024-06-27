use serde_json::Value;
use sha1::{Digest, Sha1};
use crate::domain::entities::TorrentInfo;

pub fn extract_torrent_info(decoded_value: &Value) -> TorrentInfo
{
    let announce = extract_announce(decoded_value);
    let info = extract_info(decoded_value);

    let length = extract_length(&info);
    let piece_length = extract_piece_length(&info);
    let pieces = extract_pieces(&info);
    let info_hash = extract_info_hash(&info);

    TorrentInfo
    {
        announce,
        length,
        piece_length,
        pieces,
        info_hash,
    }
}

fn extract_announce(decoded_value: &Value) -> Option<String>
{
    decoded_value.get("announce").and_then(|v| v.as_str()).map(String::from)
}

fn extract_info(decoded_value: &Value) -> Option<&serde_json::Map<String, Value>>
{
    decoded_value.get("info").and_then(|v| v.as_object())
}

fn extract_length(info: &Option<&serde_json::Map<String, Value>>) -> Option<i64>
{
    info.and_then(|i| i.get("length")).and_then(|v| v.as_i64())
}

fn extract_piece_length(info: &Option<&serde_json::Map<String, Value>>) -> Option<i64>
{
    info.and_then(|i| i.get("piece length")).and_then(|v| v.as_i64())
}

fn extract_pieces(info: &Option<&serde_json::Map<String, Value>>) -> Option<Vec<String>>
{
    info.and_then(|i| i.get("pieces")).and_then(|v| v.as_str()).map(|s|
    {
        s.as_bytes().chunks(20)
            .map(|chunk| format!("{:x}", Sha1::digest(chunk)))
            .collect()
    })
}

fn extract_info_hash(info: &Option<&serde_json::Map<String, Value>>) -> Option<String>
{
    info.map(|i|
    {
        let bencoded_info = bencode_info_dict(i);
        format!("{:x}", Sha1::digest(&bencoded_info))
    })
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
        Value::Number(n) if n.is_i64() => bencode_number(buffer, n),
        Value::String(s) => bencode_string(buffer, s),
        Value::Array(arr) => bencode_array(buffer, arr),
        Value::Object(obj) => bencode_object(buffer, obj),
        _ => panic!("Unsupported value type for bencoding"),
    }
}

fn bencode_number(buffer: &mut Vec<u8>, n: &serde_json::Number)
{
    buffer.push(b'i');
    buffer.extend_from_slice(n.to_string().as_bytes());
    buffer.push(b'e');
}

fn bencode_array(buffer: &mut Vec<u8>, arr: &Vec<Value>)
{
    buffer.push(b'l');
    for v in arr
    {
        bencode_value(buffer, v);
    }
    buffer.push(b'e');
}

fn bencode_object(buffer: &mut Vec<u8>, obj: &serde_json::Map<String, Value>)
{
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

