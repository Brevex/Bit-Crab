use uuid::Uuid;
use std::net::SocketAddr;
use std::convert::TryInto;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::{bail, Context, Result};

const RESERVED_BYTES: [u8; 8] = [0; 8];
const PROTOCOL_STRING: &str = "BitTorrent protocol";

pub async fn perform_handshake(peer_addr: &SocketAddr, info_hash: &[u8]) -> Result<[u8; 20]>
{
    if info_hash.len() != 20
    {
        bail!("info hash must have 20 bytes");
    }

    let peer_id = generate_peer_id();
    println!("Generated Peer ID: {}", hex::encode(&peer_id));

    let mut stream = TcpStream::connect(peer_addr)
        .await
        .context("Failed to connect to peer")?;

    send_handshake(&mut stream, info_hash, &peer_id).await?;
    let received_peer_id = receive_handshake(&mut stream).await?;

    println!("Received Peer ID: {}", hex::encode(received_peer_id));
    Ok(received_peer_id)
}

async fn send_handshake(stream: &mut TcpStream, info_hash: &[u8], peer_id: &[u8]) -> Result<()>
{
    let mut handshake_msg = Vec::with_capacity(68);
    handshake_msg.push(PROTOCOL_STRING.len() as u8);
    handshake_msg.extend_from_slice(PROTOCOL_STRING.as_bytes());
    handshake_msg.extend_from_slice(&RESERVED_BYTES);
    handshake_msg.extend_from_slice(info_hash);
    handshake_msg.extend_from_slice(peer_id);

    stream.write_all(&handshake_msg)
        .await
        .context("Failed to send handshake")
}

async fn receive_handshake(stream: &mut TcpStream) -> Result<[u8; 20]>
{
    let mut response = [0u8; 68];
    stream.read_exact(&mut response)
        .await
        .context("Failed to read handshake response")?;

    if response[0] as usize != PROTOCOL_STRING.len()
        || &response[1..1 + PROTOCOL_STRING.len()] != PROTOCOL_STRING.as_bytes()
    {
        bail!("Invalid handshake response");
    }
    let received_peer_id: [u8; 20] = response[48..68]
        .try_into()
        .context("Failed to extract peer ID from response")?;

    Ok(received_peer_id)
}

fn generate_peer_id() -> [u8; 20]
{
    let uuid = Uuid::new_v4();
    let mut peer_id = [0u8; 20];
    
    peer_id[..16].copy_from_slice(uuid.as_bytes());
    peer_id[16..].copy_from_slice(&[0u8; 4]);
    peer_id
}
