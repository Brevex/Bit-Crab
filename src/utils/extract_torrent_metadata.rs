use crate::entities::torrent::FileInfo;
use crate::utils::errors::MetadataError;

use anyhow::Result;
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use serde_bencode::value::Value;
use std::collections::HashMap;

type BencodeDict = HashMap<Vec<u8>, Value>;
type BencodeList = Vec<Value>;

pub fn extract_string(key: &str, dict: &BencodeDict) -> Result<String, MetadataError>
{
    if let Some(Value::Bytes(b)) = dict.get(key.as_bytes())
    {
        String::from_utf8(b.clone()).map_err(|_| MetadataError::FieldError(key.to_string()))
    }
    else { Err(MetadataError::FieldError(key.to_string())) }
}

pub fn extract_dict(key: &str, dict: &BencodeDict) -> Result<BencodeDict, MetadataError>
{
    if let Some(Value::Dict(d)) = dict.get(key.as_bytes())
    {
        Ok(d.clone())
    }
    else { Err(MetadataError::FieldError(key.to_string())) }
}

pub fn extract_list(key: &str, dict: &BencodeDict) -> Result<BencodeList, MetadataError>
{
    if let Some(Value::List(l)) = dict.get(key.as_bytes())
    {
        Ok(l.clone())
    }
    else { Err(MetadataError::FieldError(key.to_string())) }
}

pub fn extract_int(key: &str, dict: &BencodeDict) -> Result<i64, MetadataError>
{
    if let Some(Value::Int(i)) = dict.get(key.as_bytes())
    {
        Ok(*i)
    }
    else { Err(MetadataError::FieldError(key.to_string())) }
}

pub fn extract_bytes(key: &str, dict: &BencodeDict) -> Result<Vec<u8>, MetadataError>
{
    if let Some(Value::Bytes(b)) = dict.get(key.as_bytes())
    {
        Ok(b.clone())
    }
    else { Err(MetadataError::FieldError(key.to_string())) }
}

pub fn extract_files(info: &BencodeDict) -> Result<Vec<FileInfo>, MetadataError>
{
    let files = extract_list("files", info)?;

    files
        .into_iter()
        .map(|value| {
            if let Value::Dict(file_info) = value
            {
                let length = extract_int("length", &file_info)?;
                let path = extract_list("path", &file_info)?
                    .into_iter()
                    .filter_map(|v| {
                        if let Value::Bytes(b) = v
                        {
                            String::from_utf8(b).ok()
                        }
                        else { None }
                    })
                    .collect();
                Ok(FileInfo::new(length, path))
            }
            else { Err(MetadataError::IncorrectFormatError) }
        })
        .collect()
}

pub fn generate_peer_id() -> String
{
    Alphanumeric.sample_string(&mut thread_rng(), 20)
}
