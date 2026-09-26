pub struct Crc {
    crc: u32,
    digest_step: u32,
    mask: u32,
}

impl Crc {
    pub fn new_normal(hh: u16, rr: u16) -> Self {
        Self {
            digest_step: Self::get_digest_step(hh, rr),
            mask: 0xc37b,
            crc: 0xc387,
        }
    }

    fn get_digest_step(final_a: u16, final_b: u16) -> u32 {
        0xFFFF0000 | ((final_b as u32) << 8) | (final_a as u32)
    }

    pub fn step(&mut self, mut value: u32) {
        for _ in 0..32 {
            let bit = (self.crc ^ value) & 1;

            self.crc >>= 1;
            if bit != 0 {
                self.crc ^= self.mask;
            }

            value >>= 1;
        }
    }

    pub fn digest(&mut self) -> u16 {
        self.step(self.digest_step);
        self.crc as u16
    }
}

pub struct EncryptState {
    seed: u32,
    mask: u32,
}

impl EncryptState {
    pub fn new_normal(seed: u32) -> Self {
        Self {
            seed,
            mask: 0x43202F2F,
        }
    }

    pub fn step(&mut self, value: u32, i: u32) -> u32 {
        self.seed = self.seed.wrapping_mul(0x6F646573).wrapping_add(1);
        self.seed ^ value ^ 0xFE000000u32.wrapping_sub(i) ^ self.mask
    }
}
