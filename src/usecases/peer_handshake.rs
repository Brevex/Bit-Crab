use uuid::Uuid;
use std::net::SocketAddr;
use std::convert::TryInto;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::Write;
use sha1::{Sha1, Digest};
use crate::domain::entities::TorrentInfo;
use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

const RESERVED_BYTES: [u8; 8] = [0; 8];
const PROTOCOL_STRING: &str = "BitTorrent protocol";
const BLOCK_SIZE: usize = 16 * 1024; // 16 kiB

#[repr(C)]
pub struct Piece<T: ?Sized = [u8]> {
    index: [u8; 4],
    begin: [u8; 4],
    block: T,
}

impl Piece {
    pub fn index(&self) -> u32 {
        u32::from_be_bytes(self.index)
    }

    pub fn begin(&self) -> u32 {
        u32::from_be_bytes(self.begin)
    }

    pub fn block(&self) -> &[u8] {
        &self.block
    }

    const PIECE_LEAD: usize = std::mem::size_of::<Piece<()>>();

    pub fn ref_from_bytes(data: &[u8]) -> Option<&Self> {
        if data.len() < Self::PIECE_LEAD {
            return None;
        }
        let n = data.len();
        let piece = &data[..n - Self::PIECE_LEAD] as *const [u8] as *const Piece;
        Some(unsafe { &*piece })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageTag {
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub tag: MessageTag,
    pub payload: Vec<u8>,
}

pub struct MessageFramer;
const MAX: usize = 1 << 16;

impl Decoder for MessageFramer {
    type Item = Message;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_be_bytes(length_bytes) as usize;
        if length == 0 {
            src.advance(4);
            return self.decode(src);
        }
        if src.len() < 5 {
            return Ok(None);
        }

        if length > MAX {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", length),
            ));
        }
        if src.len() < 4 + length {
            src.reserve(4 + length - src.len());
            return Ok(None);
        }

        let tag = match src[4] {
            0 => MessageTag::Choke,
            1 => MessageTag::Unchoke,
            2 => MessageTag::Interested,
            3 => MessageTag::NotInterested,
            4 => MessageTag::Have,
            5 => MessageTag::Bitfield,
            6 => MessageTag::Request,
            7 => MessageTag::Piece,
            8 => MessageTag::Cancel,
            tag => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unknown message type {}.", tag),
                ))
            }
        };

        let data = if src.len() > 5 {
            src[5..4 + length].to_vec()
        } else {
            Vec::new()
        };
        src.advance(4 + length);
        Ok(Some(Message { tag, payload: data }))
    }
}

impl Encoder<Message> for MessageFramer {
    type Error = std::io::Error;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        if item.payload.len() + 1 > MAX {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", item.payload.len()),
            ));
        }

        let len_slice = u32::to_be_bytes(item.payload.len() as u32 + 1);
        dst.reserve(4 + 1 + item.payload.len());
        dst.extend_from_slice(&len_slice);
        dst.put_u8(item.tag as u8);
        dst.extend_from_slice(&item.payload);
        Ok(())
    }
}

pub async fn download_piece(peer_addr: &SocketAddr, info_hash: &[u8], piece_index: u32, piece_length: usize, torrent_info: &TorrentInfo) -> Result<()> {
    if info_hash.len() != 20 {
        bail!("info hash must have 20 bytes");
    }

    let peer_id = generate_peer_id();
    let mut stream = TcpStream::connect(peer_addr)
        .await
        .context("Failed to connect to peer")?;

    send_handshake(&mut stream, info_hash, &peer_id).await?;
    receive_handshake(&mut stream).await?;

    let bitfield = receive_message(&mut stream).await?;
    if bitfield[0] != 5 {
        bail!("Expected bitfield message");
    }

    send_interested(&mut stream).await?;

    let unchoke = receive_message(&mut stream).await?;
    if unchoke[0] != 1 {
        bail!("Expected unchoke message");
    }

    let total_length = torrent_info.length.unwrap() as usize;
    let last_piece_length = total_length % piece_length;
    let actual_piece_length = if piece_index as usize == total_length / piece_length && last_piece_length != 0 {
        last_piece_length
    } else {
        piece_length
    };

    let mut piece_data = Vec::with_capacity(actual_piece_length);
    let num_blocks = (actual_piece_length + BLOCK_SIZE - 1) / BLOCK_SIZE;

    for block_index in 0..num_blocks {
        let begin = block_index * BLOCK_SIZE;
        let length = std::cmp::min(BLOCK_SIZE, actual_piece_length - begin);

        send_request(&mut stream, piece_index, begin as u32, length as u32).await?;
        let block = match receive_piece(&mut stream, piece_index, begin as u32, length).await {
            Ok(block) => block,
            Err(e) => {
                eprintln!("Error receiving piece: {:?}", e);
                bail!("Invalid piece message");
            }
        };
        piece_data.extend_from_slice(&block);
    }

    if verify_piece(&piece_data, piece_index, torrent_info) {
        save_piece_to_disk(piece_index, &piece_data)?;
    } else {
        bail!("Piece hash does not match");
    }

    Ok(())
}

async fn send_interested(stream: &mut TcpStream) -> Result<()> {
    let msg = [0, 0, 0, 1, 2];
    stream.write_all(&msg).await.context("Failed to send interested message")
}

async fn send_request(stream: &mut TcpStream, index: u32, begin: u32, length: u32) -> Result<()> {
    let mut msg = Vec::with_capacity(17);
    msg.extend_from_slice(&(13u32.to_be_bytes()));
    msg.push(6);
    msg.extend_from_slice(&index.to_be_bytes());
    msg.extend_from_slice(&begin.to_be_bytes());
    msg.extend_from_slice(&length.to_be_bytes());
    stream.write_all(&msg).await.context("Failed to send request message")
}

async fn receive_piece(stream: &mut TcpStream, index: u32, begin: u32, length: usize) -> Result<Vec<u8>> {
    let mut length_prefix = [0u8; 4];
    if let Err(e) = stream.read_exact(&mut length_prefix).await.context("Failed to read length prefix") {
        eprintln!("Error reading length prefix: {:?}", e);
        bail!("Failed to read length prefix");
    }

    let msg_length = u32::from_be_bytes(length_prefix) as usize;
    println!("Message length: {}", msg_length);

    if msg_length != 9 + length {
        bail!("Unexpected message length: expected {}, got {}", 9 + length, msg_length);
    }

    let mut msg = vec![0u8; msg_length];
    if let Err(e) = stream.read_exact(&mut msg).await.context("Failed to read piece message") {
        eprintln!("Error reading piece message: {:?}", e);
        bail!("Failed to read piece message");
    }

    if msg[0] != 7 {
        bail!("Invalid piece message: expected message id 7, got {}", msg[0]);
    }

    if let Some(piece) = Piece::ref_from_bytes(&msg[1..]) {
        if piece.index() != index {
            bail!("Piece index mismatch: expected {}, got {}", index, piece.index());
        }
        if piece.begin() != begin {
            bail!("Piece begin mismatch: expected {}, got {}", begin, piece.begin());
        }
        Ok(piece.block().to_vec())
    } else {
        bail!("Failed to decode piece message");
    }
}

fn verify_piece(piece_data: &[u8], piece_index: u32, torrent_info: &TorrentInfo) -> bool {
    let mut hasher = Sha1::new();
    hasher.update(piece_data);
    let piece_hash = hasher.finalize();

    let expected_hash_hex = &torrent_info.pieces.as_ref().unwrap()[piece_index as usize];
    let expected_hash = hex::decode(expected_hash_hex).expect("Failed to decode expected hash");

    piece_hash.as_slice() == expected_hash.as_slice()
}

fn save_piece_to_disk(piece_index: u32, piece_data: &[u8]) -> Result<()> {
    let path = format!("/tmp/test-piece-{}.tmp", piece_index);
    let mut file = File::create(&path).context("Failed to create file")?;
    file.write_all(piece_data).context("Failed to write piece to file")?;
    println!("Piece {} downloaded to {}.", piece_index, path);
    Ok(())
}

async fn send_handshake(stream: &mut TcpStream, info_hash: &[u8], peer_id: &[u8]) -> Result<()> {
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

async fn receive_handshake(stream: &mut TcpStream) -> Result<[u8; 20]> {
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

fn generate_peer_id() -> [u8; 20] {
    let uuid = Uuid::new_v4();
    let mut peer_id = [0u8; 20];

    peer_id[..16].copy_from_slice(uuid.as_bytes());
    peer_id[16..].copy_from_slice(&[0u8; 4]);
    peer_id
}

async fn receive_message(stream: &mut TcpStream) -> Result<Vec<u8>> {
    let mut length_prefix = [0u8; 4];
    stream.read_exact(&mut length_prefix).await.context("Failed to read message length")?;
    let length = u32::from_be_bytes(length_prefix) as usize;
    let mut msg = vec![0u8; length];
    stream.read_exact(&mut msg).await.context("Failed to read message")?;
    Ok(msg)
}
