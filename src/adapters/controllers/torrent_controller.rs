use serde_json::Value;
use crate::domain::errors::TorrentError;
use crate::usecases::extract_torrent_info::extract_torrent_info;
use crate::usecases::read_file::read_file;
use crate::adapters::presenters::torrent_presenter::print_torrent_info;

pub fn handle_torrent(file_path: &str) -> Result<(), TorrentError>
{
    let buffer = read_file(file_path)?;

    let decoded_value = decode_bencoded_value(&buffer)
        .map_err(|e| TorrentError::DecodeError(format!("Failed to decode bencoded value: {}", e)))?;

    let torrent_info = extract_torrent_info(&decoded_value.0);
    print_torrent_info(&torrent_info);

    Ok(())
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

    if !rest.is_empty() && rest[0] == b'e'
    {
        Ok((values.into(), &rest[1..]))
    }
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

    if !rest.is_empty() && rest[0] == b'e'
    {
        Ok((dict.into(), &rest[1..]))
    }
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

            if encoded_value.len() >= end {
                return Ok((
                    String::from_utf8_lossy(&encoded_value[start..end]).into(),
                    &encoded_value[end..],
                ));
            }
        }
    }
    Err("Invalid string encoding".to_string())
}
