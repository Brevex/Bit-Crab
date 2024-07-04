pub struct Piece {
    index: u32,
    begin: u32,
    block: Vec<u8>,
}

impl Piece
{
    pub fn index(&self) -> u32 { self.index }
    pub fn begin(&self) -> u32 { self.begin }
    pub fn block(&self) -> &[u8] { &self.block }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 8 { return None; }

        let index = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let begin = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let block = data[8..].to_vec();

        Some(Self {
            index,
            begin,
            block,
        })
    }
}
