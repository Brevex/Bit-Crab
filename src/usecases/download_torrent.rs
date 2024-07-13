use crate::entities::message::Message;
use crate::entities::peer::Peer;
use crate::entities::torrent::Torrent;
use crate::utils::errors::{HandshakeError, TorrentError};

use anyhow::Result;
use sha1::{Digest, Sha1};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio::task;

const BLOCK_SIZE: usize = 16 * 1024;
const NUM_WORKERS: usize = 4;

pub async fn download_torrent(torrent: &Torrent, peers: &[Peer]) -> Result<(), TorrentError>
{
    let piece_length = *torrent.info().piece_length() as usize;
    let num_pieces = torrent.info().piece_hashes().len();
    let work_queue = Arc::new(Mutex::new(VecDeque::from(
        (0..num_pieces).collect::<Vec<_>>(),
    )));
    let (tx, mut rx) = mpsc::channel(NUM_WORKERS);

    for _ in 0..NUM_WORKERS
    {
        let work_queue = Arc::clone(&work_queue);
        let torrent = torrent.clone();
        let peers = peers.to_vec();
        let tx = tx.clone();

        task::spawn(async move {
            while let Some(piece_index) = get_work(&work_queue).await
            {
                if let Err(_) = download_and_verify_piece(&torrent, &peers, piece_index).await
                {
                    add_work(&work_queue, piece_index).await;
                }
                else
                {
                    tx.send(Some(piece_index)).await.unwrap();
                }
            }
            tx.send(None).await.unwrap();
        });
    }

    let mut pieces_downloaded = vec![false; num_pieces];
    let final_file_name = format!("{}", torrent.info().name());
    let mut file = File::create(&final_file_name)
        .await
        .expect("Failed to create file");
    let mut completed_workers = 0;

    while completed_workers < NUM_WORKERS
    {
        if let Some(piece_index) = rx.recv().await
        {
            if let Some(piece_index) = piece_index
            {
                pieces_downloaded[piece_index] = true;
                for i in 0..num_pieces
                {
                    if !pieces_downloaded[i] { break; }

                    let piece_file = format!("piece_{}.data", i);

                    match File::open(&piece_file).await
                    {
                        Ok(mut piece) => {
                            let mut buffer = vec![0; piece_length];
                            let bytes_read = piece
                                .read(&mut buffer)
                                .await
                                .expect("Failed to read piece file");
                            file.seek(SeekFrom::End(0))
                                .await
                                .expect("Failed to seek file");
                            file.write_all(&buffer[..bytes_read])
                                .await
                                .expect("Failed to write piece to file");
                            tokio::fs::remove_file(piece_file)
                                .await
                                .expect("Failed to delete piece file");
                        }
                        Err(e) => { eprintln!("Failed to open piece file: {:?}", e); }
                    }
                }
            }
            else { completed_workers += 1; }
        }
    }
    println!("Arquivo {} está pronto", final_file_name);
    Ok(())
}

async fn get_work(work_queue: &Arc<Mutex<VecDeque<usize>>>) -> Option<usize>
{
    let mut queue = work_queue.lock().await;
    queue.pop_front()
}

async fn add_work(work_queue: &Arc<Mutex<VecDeque<usize>>>, piece_index: usize)
{
    let mut queue = work_queue.lock().await;
    queue.push_back(piece_index);
}

async fn download_and_verify_piece(
    torrent: &Torrent,
    peers: &[Peer],
    piece_index: usize,
) -> Result<(), TorrentError>
{
    for peer in peers
    {
        match download_piece(torrent, peer, piece_index as u32).await
        {
            Ok(piece) => {
                let piece_hash = torrent.info().piece_hashes()[piece_index];
                let mut hasher = Sha1::new();
                hasher.update(&piece);
                let hash = hasher.finalize();

                if hash.as_slice() == piece_hash
                {
                    let piece_file = format!("piece_{}.data", piece_index);
                    let mut file = File::create(&piece_file)
                        .await
                        .expect("Failed to create piece file");
                    file.write_all(&piece)
                        .await
                        .expect("Failed to write piece to file");
                    return Ok(());
                }
            }
            Err(_) => { continue; }
        }
    }
    Err(TorrentError::IoError(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Failed to download piece",
    )))
}

pub async fn download_piece(
    torrent: &Torrent,
    peer: &Peer,
    piece_index: u32,
) -> Result<Vec<u8>, TorrentError>
{
    let addr = format!("{}:{}", peer.ip(), peer.port());
    let mut stream = TcpStream::connect(&addr)
        .await
        .map_err(|_| HandshakeError::ConnectionError(addr.clone()))?;
    let handshake = crate::entities::handshake::Handshake::new(*torrent.info_hash());
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
        return Err(HandshakeError::InvalidHandshakeResponse(addr).into());
    }

    let bitfield_message = read_message(&mut stream).await?;
    let _bitfield = if let Message::Bitfield { bitfield } = bitfield_message {
        bitfield
    }
    else
    {
        return Err(TorrentError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Esperava mensagem bitfield",
        )));
    };

    let interested_message = Message::Interested;
    stream.write_all(&interested_message.as_bytes()).await?;

    let unchoke_message = read_message(&mut stream).await?;
    if !matches!(unchoke_message, Message::Unchoke)
    {
        return Err(TorrentError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Esperava mensagem unchoke",
        )));
    }

    let piece_length = *torrent.info().piece_length() as usize;
    let mut buffer = vec![];
    let mut offset = 0;

    while offset < piece_length
    {
        let block_size = std::cmp::min(BLOCK_SIZE, piece_length - offset);
        let request_message = Message::Request {
            index: piece_index,
            begin: offset as u32,
            length: block_size as u32,
        };
        stream.write_all(&request_message.as_bytes()).await?;
        let piece_message = read_message(&mut stream).await?;

        if let Message::Piece { block, .. } = piece_message
        {
            buffer.extend_from_slice(&block);
        }
        else
        {
            return Err(TorrentError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Esperava mensagem piece",
            )));
        }
        offset += block_size;
    }
    Ok(buffer)
}

async fn read_message(stream: &mut TcpStream) -> Result<Message, TorrentError>
{
    let mut length_prefix = [0; 4];
    stream.read_exact(&mut length_prefix).await?;
    let length_prefix = u32::from_be_bytes(length_prefix);

    let mut buffer = vec![0; length_prefix as usize];
    stream.read_exact(&mut buffer).await?;

    Message::from_bytes(&buffer).ok_or_else(|| {
        TorrentError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to parse message",
        ))
    })
}
