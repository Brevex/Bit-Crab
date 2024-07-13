use crate::entities::handshake::Handshake;
use crate::entities::peer::Peer;
use crate::entities::torrent::Torrent;
use crate::utils::errors::{HandshakeError, TorrentError};

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

pub async fn perform_handshake(
    torrent: &Torrent,
    peers: &[Peer],
) -> Result<Vec<Peer>, TorrentError>
{
    let mut connected_peers = Vec::new();
    let handshake = Handshake::new(*torrent.info_hash());

    for peer in peers
    {
        format!("{}:{}", peer.ip(), peer.port());
        match timeout(Duration::from_secs(5), try_handshake(&handshake, peer)).await
        {
            Ok(Ok(())) => {
                println!(
                    "Handshake successfully performed with peer: {}:{}",
                    peer.ip(),
                    peer.port()
                );
                connected_peers.push(peer.clone());
            }
            Ok(Err(e)) => {
                eprintln!(
                    "Handshake failed with peer {}:{} - Error: {}",
                    peer.ip(),
                    peer.port(),
                    e
                );
            }
            Err(_) => {
                eprintln!(
                    "Handshake timed out with peer {}:{}",
                    peer.ip(),
                    peer.port()
                );
            }
        }
    }
    Ok(connected_peers)
}

async fn try_handshake(handshake: &Handshake, peer: &Peer) -> Result<(), HandshakeError>
{
    let addr = format!("{}:{}", peer.ip(), peer.port());
    let mut stream = TcpStream::connect(&addr)
        .await
        .map_err(|_| HandshakeError::ConnectionError(addr.clone()))?;
    stream
        .write_all(&handshake.as_bytes())
        .await
        .map_err(|_| HandshakeError::HandshakeSendError(addr.clone()))?;

    let mut response = vec![0; 68];
    stream
        .read_exact(&mut response)
        .await
        .map_err(|_| HandshakeError::HandshakeReceiveError(addr.clone()))?;

    if &response[1..20] != handshake.protocol_str().as_bytes()
        || &response[28..48] != handshake.info_hash()
    {
        return Err(HandshakeError::InvalidHandshakeResponse(addr));
    }
    Ok(())
}
