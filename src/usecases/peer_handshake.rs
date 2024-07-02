use std::convert::TryInto;
use std::fs::File;
use std::io::Write;
use std::net::SocketAddr;

use anyhow::{bail, Context, Result};
use sha1::{Digest, Sha1};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::domain::entities::info::TorrentInfo;
use crate::domain::entities::piece::Piece;

const RESERVED_BYTES: [u8; 8] = [0; 8];
const PROTOCOL_STRING: &str = "BitTorrent protocol";
const BLOCK_SIZE: usize = 16 * 1024;

pub(crate) async fn download_piece(
    peer_addr: &SocketAddr,
    info_hash: &[u8; 20],
    piece_index: u32,
    piece_length: usize,
    torrent_info: &TorrentInfo,
) -> Result<()>
{
    let peer_id = generate_peer_id();
    let mut stream = connect_to_peer(peer_addr).await?;
    perform_handshake(&mut stream, info_hash, &peer_id).await?;
    ensure_interested_and_unchoked(&mut stream).await?;

    let piece_data =
        download_piece_data(&mut stream, piece_index, piece_length, torrent_info).await?;

    if verify_piece(&piece_data, piece_index, torrent_info)
    {
        save_piece_to_disk(piece_index, &piece_data)?;
    }
    else { bail!("Piece hash does not match"); }
    Ok(())
}

async fn connect_to_peer(peer_addr: &SocketAddr) -> Result<TcpStream>
{
    TcpStream::connect(peer_addr)
        .await
        .context("Failed to connect to peer")
}

async fn perform_handshake(
    stream: &mut TcpStream,
    info_hash: &[u8],
    peer_id: &String,
) -> Result<()>
{
    send_handshake(stream, info_hash, peer_id).await?;
    receive_handshake(stream).await?;
    Ok(())
}

async fn ensure_interested_and_unchoked(stream: &mut TcpStream) -> Result<()>
{
    let bitfield = receive_message(stream).await?;
    if bitfield[0] != 5
    {
        bail!("Expected bitfield message");
    }

    send_interested(stream).await?;

    let unchoke = receive_message(stream).await?;
    if unchoke[0] != 1
    {
        bail!("Expected unchoke message");
    }
    Ok(())
}

async fn download_piece_data(
    stream: &mut TcpStream,
    piece_index: u32,
    piece_length: usize,
    torrent_info: &TorrentInfo,
) -> Result<Vec<u8>>
{
    let total_length = torrent_info.length.unwrap() as usize;
    let last_piece_length = total_length % piece_length;
    let actual_piece_length =
        if piece_index as usize == total_length / piece_length && last_piece_length != 0
        {
            last_piece_length
        }
        else { piece_length };

    let mut piece_data = Vec::with_capacity(actual_piece_length);
    let num_blocks = (actual_piece_length + BLOCK_SIZE - 1) / BLOCK_SIZE;

    for block_index in 0..num_blocks
    {
        let begin = block_index * BLOCK_SIZE;
        let length = std::cmp::min(BLOCK_SIZE, actual_piece_length - begin);

        send_request(stream, piece_index, begin as u32, length as u32).await?;
        let block = receive_piece(stream, piece_index, begin as u32, length).await?;
        piece_data.extend_from_slice(&block);
    }

    Ok(piece_data)
}

async fn send_interested(stream: &mut TcpStream) -> Result<()>
{
    let msg = [0, 0, 0, 1, 2];

    stream
        .write_all(&msg)
        .await
        .context("Failed to send interested message")
}

async fn send_request(stream: &mut TcpStream, index: u32, begin: u32, length: u32) -> Result<()>
{
    let mut msg = Vec::with_capacity(17);

    msg.extend_from_slice(&(13u32.to_be_bytes()));
    msg.push(6);
    msg.extend_from_slice(&index.to_be_bytes());
    msg.extend_from_slice(&begin.to_be_bytes());
    msg.extend_from_slice(&length.to_be_bytes());

    stream
        .write_all(&msg)
        .await
        .context("Failed to send request message")
}

async fn receive_piece(
    stream: &mut TcpStream,
    index: u32,
    begin: u32,
    length: usize,
) -> Result<Vec<u8>>
{
    let mut length_prefix = [0u8; 4];

    if let Err(e) = stream
        .read_exact(&mut length_prefix)
        .await
        .context("Failed to read length prefix")
    {
        eprintln!("Error reading length prefix: {:?}", e);
        bail!("Failed to read length prefix");
    }

    let msg_length = u32::from_be_bytes(length_prefix) as usize;
    println!("Message length: {}", msg_length);

    if msg_length != 9 + length
    {
        bail!(
            "Unexpected message length: \
        expected {}, \
        got {}",
            9 + length,
            msg_length
        );
    }

    let mut msg = vec![0u8; msg_length];

    if let Err(e) = stream
        .read_exact(&mut msg)
        .await
        .context("Failed to read piece message")
    {
        eprintln!("Error reading piece message: {:?}", e);
        bail!("Failed to read piece message");
    }

    if msg[0] != 7
    {
        bail!(
            "Invalid piece message: \
        expected message id 7, \
        got {}",
            msg[0]
        );
    }

    if let Some(piece) = Piece::ref_from_bytes(&msg[1..])
    {
        if piece.index() != index
        {
            bail!(
                "Piece index mismatch: \
            expected {}, \
            got {}",
                index,
                piece.index()
            );
        }
        if piece.begin() != begin
        {
            bail!(
                "Piece begin mismatch: \
            expected {}, got {}",
                begin,
                piece.begin()
            );
        }
        Ok(piece.block().to_vec())
    }
    else { bail!("Failed to decode piece message"); }
}

fn verify_piece(piece_data: &[u8], piece_index: u32, torrent_info: &TorrentInfo) -> bool
{
    let mut hasher = Sha1::new();
    hasher.update(piece_data);

    let piece_hash = hasher.finalize();
    let expected_hash = torrent_info.pieces.as_ref().unwrap().to_hash_vec()[piece_index as usize];

    piece_hash.as_slice() == expected_hash.as_slice()
}

fn save_piece_to_disk(piece_index: u32, piece_data: &[u8]) -> Result<()>
{
    let path = format!("/tmp/test-piece-{}.tmp", piece_index);
    let mut file = File::create(&path).context("Failed to create file")?;

    file.write_all(piece_data)
        .context("Failed to write piece to file")?;
    println!("Piece {} downloaded to {}.", piece_index, path);

    Ok(())
}

async fn send_handshake(stream: &mut TcpStream, info_hash: &[u8], peer_id: &String) -> Result<()>
{
    let mut handshake_msg = Vec::with_capacity(68);

    handshake_msg.push(PROTOCOL_STRING.len() as u8);
    handshake_msg.extend_from_slice(PROTOCOL_STRING.as_bytes());
    handshake_msg.extend_from_slice(&RESERVED_BYTES);
    handshake_msg.extend_from_slice(info_hash);
    handshake_msg.extend_from_slice(peer_id.as_ref());

    stream
        .write_all(&handshake_msg)
        .await
        .context("Failed to send handshake")
}

async fn receive_handshake(stream: &mut TcpStream) -> Result<[u8; 20]>
{
    let mut response = [0u8; 68];
    stream
        .read_exact(&mut response)
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

async fn receive_message(stream: &mut TcpStream) -> Result<Vec<u8>>
{
    let mut length_prefix = [0u8; 4];

    stream
        .read_exact(&mut length_prefix)
        .await
        .context("Failed to read message length")?;

    let length = u32::from_be_bytes(length_prefix) as usize;
    let mut msg = vec![0u8; length];

    stream
        .read_exact(&mut msg)
        .await
        .context("Failed to read message")?;

    Ok(msg)
}

fn generate_peer_id() -> String
{
    crate::usecases::utils::generate_peer_id::generate_peer_id()
}
