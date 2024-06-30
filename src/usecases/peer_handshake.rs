use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::convert::TryInto;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

const PROTOCOL_STRING: &str = "BitTorrent protocol";
const RESERVED_BYTES: [u8; 8] = [0; 8];

pub async fn perform_handshake(
    peer_addr: &SocketAddr,
    info_hash: &[u8]) -> Result<[u8; 20]>
{
    if info_hash.len() != 20
    {
        anyhow::bail!("info hash must have 20 bytes");
    }

    let peer_id = generate_peer_id();

    let mut stream = TcpStream::connect(peer_addr)
        .await
        .context("Failed to connect to peer")?;

    let mut handshake_msg = Vec::with_capacity(68);

    handshake_msg.push(PROTOCOL_STRING.len() as u8);
    handshake_msg.extend_from_slice(PROTOCOL_STRING.as_bytes());
    handshake_msg.extend_from_slice(&RESERVED_BYTES);
    handshake_msg.extend_from_slice(info_hash);
    handshake_msg.extend_from_slice(peer_id.as_bytes());

    stream.write_all(&handshake_msg)
        .await
        .context("Failed to send handshake")?;

    let mut response = [0u8; 68];
    stream.read_exact(&mut response)
        .await
        .context("Failed to read handshake response")?;

    if response[0] as usize != PROTOCOL_STRING.len() ||
        &response[1..1 + PROTOCOL_STRING.len()] != PROTOCOL_STRING.as_bytes() {
        anyhow::bail!("Invalid handshake response");
    }

    let received_peer_id: [u8; 20] = response[48..68]
        .try_into()
        .context("Failed to extract peer ID from response")?;

    println!("Received Peer ID: {}", hex::encode(received_peer_id));
    Ok(received_peer_id)
}

fn generate_peer_id() -> String
{
    let mut rng = thread_rng();
    let peer_id: String = (&mut rng)
        .sample_iter(Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();

    peer_id
}