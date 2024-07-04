use bytes::{Buf, BufMut, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

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
    Port = 9,
}

#[derive(Debug, Clone)]
pub enum Message {
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Bitfield(Vec<u8>),
    Request {
        index: u32,
        begin: u32,
        length: u32,
    },
    Piece {
        index: u32,
        begin: u32,
        block: Vec<u8>,
    },
    Cancel {
        index: u32,
        begin: u32,
        length: u32,
    },
    Port(u16),
}

pub struct MessageFramer;

impl Decoder for MessageFramer {
    type Item = Message;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        let length = src.get_u32() as usize;

        if src.len() < length {
            src.reserve(length - src.len());
            return Ok(None);
        }

        let tag = src.get_u8();
        let message = match tag {
            0 => Message::Choke,
            1 => Message::Unchoke,
            2 => Message::Interested,
            3 => Message::NotInterested,
            4 => {
                if src.len() < 4 {
                    return Ok(None);
                }
                let piece_index = src.get_u32();
                Message::Have(piece_index)
            }
            5 => {
                let bitfield = src.split_to(length - 1).to_vec();
                Message::Bitfield(bitfield)
            }
            6 => {
                if src.len() < 12 {
                    return Ok(None);
                }
                let index = src.get_u32();
                let begin = src.get_u32();
                let length = src.get_u32();
                Message::Request {
                    index,
                    begin,
                    length,
                }
            }
            7 => {
                if src.len() < 8 {
                    return Ok(None);
                }
                let index = src.get_u32();
                let begin = src.get_u32();
                let block = src.split_to(length - 9).to_vec();
                Message::Piece {
                    index,
                    begin,
                    block,
                }
            }
            8 => {
                if src.len() < 12 {
                    return Ok(None);
                }
                let index = src.get_u32();
                let begin = src.get_u32();
                let length = src.get_u32();
                Message::Cancel {
                    index,
                    begin,
                    length,
                }
            }
            9 => {
                if src.len() < 2 {
                    return Ok(None);
                }
                let listen_port = src.get_u16();
                Message::Port(listen_port)
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unknown message type",
                ))
            }
        };
        Ok(Some(message))
    }
}

impl Encoder<Message> for MessageFramer {
    type Error = io::Error;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            Message::Choke => {
                dst.put_u32(1);
                dst.put_u8(MessageTag::Choke as u8);
            }
            Message::Unchoke => {
                dst.put_u32(1);
                dst.put_u8(MessageTag::Unchoke as u8);
            }
            Message::Interested => {
                dst.put_u32(1);
                dst.put_u8(MessageTag::Interested as u8);
            }
            Message::NotInterested => {
                dst.put_u32(1);
                dst.put_u8(MessageTag::NotInterested as u8);
            }
            Message::Have(index) => {
                dst.put_u32(5);
                dst.put_u8(MessageTag::Have as u8);
                dst.put_u32(index);
            }
            Message::Bitfield(bitfield) => {
                dst.put_u32(1 + bitfield.len() as u32);
                dst.put_u8(MessageTag::Bitfield as u8);
                dst.put_slice(&bitfield);
            }
            Message::Request {
                index,
                begin,
                length,
            } => {
                dst.put_u32(13);
                dst.put_u8(MessageTag::Request as u8);
                dst.put_u32(index);
                dst.put_u32(begin);
                dst.put_u32(length);
            }
            Message::Piece {
                index,
                begin,
                block,
            } => {
                dst.put_u32(9 + block.len() as u32);
                dst.put_u8(MessageTag::Piece as u8);
                dst.put_u32(index);
                dst.put_u32(begin);
                dst.put_slice(&block);
            }
            Message::Cancel {
                index,
                begin,
                length,
            } => {
                dst.put_u32(13);
                dst.put_u8(MessageTag::Cancel as u8);
                dst.put_u32(index);
                dst.put_u32(begin);
                dst.put_u32(length);
            }
            Message::Port(port) => {
                dst.put_u32(3);
                dst.put_u8(MessageTag::Port as u8);
                dst.put_u16(port);
            }
        }
        Ok(())
    }
}
