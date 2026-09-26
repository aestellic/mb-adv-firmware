pub struct Rom<'a> {
    bytes: &'a [u8],
}

impl<'a> Rom<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn aligned_game_len(&self) -> u32 {
        (self.bytes.len() as u32) & !0xf
    }

    pub fn header(&self) -> &[u8] {
        &self.bytes[..0xc0]
    }

    pub fn game(&self) -> &[u8] {
        let aligned_len = self.aligned_game_len() as usize;
        &self.bytes[0xc0..aligned_len]
    }
}
