#[derive(Debug)]
pub enum Message
{
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have
    {
        piece_index: u32,
    },
    Bitfield
    {
        bitfield: Vec<u8>,
    },
    Request
    {
        index: u32,
        begin: u32,
        length: u32,
    },
    Piece
    {
        index: u32,
        begin: u32,
        block: Vec<u8>,
    },
    Cancel
    {
        index: u32,
        begin: u32,
        length: u32,
    },
    Port
    {
        listen_port: u16,
    },
}

impl Message
{
    pub fn from_bytes(bytes: &[u8]) -> Option<Self>
    {
        if bytes.is_empty()
        {
            return None;
        }

        let id = bytes[0];
        match id
        {
            0 => Some(Message::Choke),
            1 => Some(Message::Unchoke),
            2 => Some(Message::Interested),
            3 => Some(Message::NotInterested),
            4 => {
                if bytes.len() < 5
                {
                    return None;
                }
                let piece_index = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                Some(Message::Have { piece_index })
            }
            5 => Some(Message::Bitfield {
                bitfield: bytes[1..].to_vec(),
            }),
            6 => {
                if bytes.len() < 13
                {
                    return None;
                }
                let index = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                let begin = u32::from_be_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]);
                let length = u32::from_be_bytes([bytes[9], bytes[10], bytes[11], bytes[12]]);
                Some(Message::Request {
                    index,
                    begin,
                    length,
                })
            }
            7 => {
                if bytes.len() < 13
                {
                    return None;
                }
                let index = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                let begin = u32::from_be_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]);
                let block = bytes[9..].to_vec();
                Some(Message::Piece {
                    index,
                    begin,
                    block,
                })
            }
            8 => {
                if bytes.len() < 13
                {
                    return None;
                }
                let index = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                let begin = u32::from_be_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]);
                let length = u32::from_be_bytes([bytes[9], bytes[10], bytes[11], bytes[12]]);
                Some(Message::Cancel {
                    index,
                    begin,
                    length,
                })
            }
            9 => {
                if bytes.len() < 3
                {
                    return None;
                }
                let listen_port = u16::from_be_bytes([bytes[1], bytes[2]]);
                Some(Message::Port { listen_port })
            }
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        match self
        {
            Message::Choke => vec![0, 0, 0, 1, 0],
            Message::Unchoke => vec![0, 0, 0, 1, 1],
            Message::Interested => vec![0, 0, 0, 1, 2],
            Message::NotInterested => vec![0, 0, 0, 1, 3],
            Message::Have { piece_index } => {
                let mut buf = vec![0, 0, 0, 5, 4];
                buf.extend_from_slice(&piece_index.to_be_bytes());
                buf
            }
            Message::Bitfield { bitfield } => {
                let mut buf = vec![0, 0, 0, 1 + bitfield.len() as u8, 5];
                buf.extend_from_slice(bitfield);
                buf
            }
            Message::Request {
                index,
                begin,
                length,
            } => {
                let mut buf = vec![0, 0, 0, 13, 6];
                buf.extend_from_slice(&index.to_be_bytes());
                buf.extend_from_slice(&begin.to_be_bytes());
                buf.extend_from_slice(&length.to_be_bytes());
                buf
            }
            Message::Piece {
                index,
                begin,
                block,
            } => {
                let mut buf = vec![0, 0, 0, (9 + block.len()) as u8, 7];
                buf.extend_from_slice(&index.to_be_bytes());
                buf.extend_from_slice(&begin.to_be_bytes());
                buf.extend_from_slice(block);
                buf
            }
            Message::Cancel {
                index,
                begin,
                length,
            } => {
                let mut buf = vec![0, 0, 0, 13, 8];
                buf.extend_from_slice(&index.to_be_bytes());
                buf.extend_from_slice(&begin.to_be_bytes());
                buf.extend_from_slice(&length.to_be_bytes());
                buf
            }
            Message::Port { listen_port } => {
                let mut buf = vec![0, 0, 0, 3, 9];
                buf.extend_from_slice(&listen_port.to_be_bytes());
                buf
            }
        }
    }
}
