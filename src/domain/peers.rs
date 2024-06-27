use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};
use std::fmt;
use std::net::{Ipv4Addr, SocketAddrV4};

#[derive(Debug, Clone)]
pub struct Peers(pub Vec<SocketAddrV4>);

struct PeersVisitor;

impl<'de> Visitor<'de> for PeersVisitor
{
    type Value = Peers;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result
    {
        formatter.write_str("6 bytes, the first 4 bytes are a peer's IP address and the last 2 are a peer's port number")
    }
    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if bytes.len() % 6 != 0
        {
            return Err(E::custom(format!("length is {}", bytes.len())));
        }
        let peers = bytes.chunks_exact(6)
            .map(|chunk| parse_peer(chunk))
            .collect::<Result<Vec<_>, E>>()?;
        Ok(Peers(peers))
    }
}

impl<'de> Deserialize<'de> for Peers
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(PeersVisitor)
    }
}

impl Serialize for Peers
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.0.iter()
            .flat_map(|peer| serialize_peer(peer))
            .collect::<Vec<_>>();

        serializer.serialize_bytes(&bytes)
    }
}

fn parse_peer<E>(bytes: &[u8]) -> Result<SocketAddrV4, E>
where
    E: de::Error,
{
    if bytes.len() != 6
    {
        return Err(E::custom("Each peer should be represented by 6 bytes"));
    }
    let ip = Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]);
    let port = u16::from_be_bytes([bytes[4], bytes[5]]);
    Ok(SocketAddrV4::new(ip, port))
}

fn serialize_peer(peer: &SocketAddrV4) -> Vec<u8>
{
    let mut bytes = Vec::with_capacity(6);
    bytes.extend_from_slice(&peer.ip().octets());
    bytes.extend_from_slice(&peer.port().to_be_bytes());
    bytes
}
