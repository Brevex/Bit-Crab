#[repr(C)]
pub struct Piece<T: ?Sized = [u8]>
{
    index: [u8; 4],
    begin: [u8; 4],
    block: T,
}

impl Piece
{
    pub fn index(&self) -> u32
    {
        u32::from_be_bytes(self.index)
    }
    pub fn begin(&self) -> u32
    {
        u32::from_be_bytes(self.begin)
    }
    pub fn block(&self) -> &[u8]
    {
        &self.block
    }
    const PIECE_LEAD: usize = std::mem::size_of::<Piece<()>>();

    pub fn ref_from_bytes(data: &[u8]) -> Option<&Self>
    {
        if data.len() < Self::PIECE_LEAD
        {
            return None;
        }
        let n = data.len();
        let piece = &data[..n - Self::PIECE_LEAD] as *const [u8] as *const Piece;

        Some(unsafe { &*piece })
    }
}
