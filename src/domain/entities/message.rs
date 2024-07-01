use bytes::{Buf, BufMut, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageTag
{
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
pub struct Message
{
    pub tag: MessageTag,
    pub payload: Vec<u8>,
}

pub struct MessageFramer;

impl Decoder for MessageFramer
{
    type Item = Message;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error>
    {
        if src.len() < 5
        {
            return Ok(None);
        }

        let length = src.get_u32() as usize;

        if src.len() < length
        {
            src.reserve(length - src.len());
            return Ok(None);
        }

        let tag = src.get_u8();
        let payload = src.split_to(length - 1).to_vec();

        let message_tag = match tag
        {
            0 => MessageTag::Choke,
            1 => MessageTag::Unchoke,
            2 => MessageTag::Interested,
            3 => MessageTag::NotInterested,
            4 => MessageTag::Have,
            5 => MessageTag::Bitfield,
            6 => MessageTag::Request,
            7 => MessageTag::Piece,
            8 => MessageTag::Cancel,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unknown message type",
                ))
            }
        };
        Ok(Some(Message {
            tag: message_tag,
            payload,
        }))
    }
}

impl Encoder<Message> for MessageFramer
{
    type Error = io::Error;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error>
    {
        let length = item.payload.len() + 1;

        if length > u32::MAX as usize
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Frame too large",
            ));
        }
        dst.put_u32(length as u32);
        dst.put_u8(item.tag as u8);
        dst.put_slice(&item.payload);

        Ok(())
    }
}
