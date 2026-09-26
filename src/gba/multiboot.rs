// Thanks to:
// - https://github.com/tangrs/usb-gba-multiboot
// - https://github.com/merryhime/gba-multiboot

use super::{
    crc::{Crc, EncryptState},
    rom::Rom,
    spi::GbaSpi,
};
use cortex_m::delay::Delay;

pub enum MultibootError {
    FailedHandshake,
    TransmissionError,
    InvalidChecksum,
}

const P_COLOR: u16 = 0;
const P_DIR: u16 = 0;
const P_SPEED: u16 = 0;

pub struct Gba<'a, Spi: GbaSpi> {
    spi: &'a mut Spi,
    rom: Rom<'a>,
}

impl<'a, Spi: GbaSpi> Gba<'a, Spi> {
    pub fn new(gba: &'a mut Spi, rom: &'a [u8]) -> Self {
        Self {
            spi: gba,
            rom: Rom::new(rom),
        }
    }

    pub fn is_ready(&mut self, delay: &mut Delay) -> bool {
        self.spi.send16(delay, 0x6202) == 0x7202
    }

    pub fn send_header(&mut self, delay: &mut Delay) {
        self.spi.send16(delay, 0x6100);

        self.rom.header().chunks_exact(2).for_each(|bytes| {
            let short = u16::from_le_bytes(bytes.try_into().unwrap());
            self.spi.send16(delay, short);
        });

        self.spi.send16(delay, 0x6200);
    }

    pub fn get_keys(&mut self, delay: &mut Delay) -> Result<(Crc, EncryptState), MultibootError> {
        let pp = 0x81 + P_COLOR * 0x10 + P_DIR * 0x8 + P_SPEED * 0x2;

        self.spi.send16(delay, 0x6202);
        // Send enc key
        self.spi.send16(delay, 0x6300 | pp);
        // Get enc key
        let token = self.spi.send16(delay, 0x6300 | pp);

        if token >> 8 != 0x73 {
            return Err(MultibootError::FailedHandshake);
        }

        let crc_final_a = token & 0xff;
        let seed = 0xFFFF0000 | (crc_final_a << 8) as u32 | pp as u32;
        let crc_final_a = (crc_final_a + 0xF) & 0xFF;

        self.spi.send16(delay, 0x6400 | crc_final_a);
        let token = self.spi.send16(
            delay,
            (((self.rom.aligned_game_len() - 0xc0) / 4) - 0x34) as u16,
        );
        let crc_final_b = token & 0xFF;

        let crc = Crc::new_normal(crc_final_a, crc_final_b);
        let enc = EncryptState::new_normal(seed);

        Ok((crc, enc))
    }

    pub fn send_rom(
        &mut self,
        delay: &mut Delay,
        crc: &mut Crc,
        enc: &mut EncryptState,
    ) -> Result<(), MultibootError> {
        for (i, bytes) in self.rom.game().chunks_exact(4).enumerate() {
            let index = ((i * 4) + 0xc0) as u32;
            let word = u32::from_le_bytes(bytes.try_into().unwrap());
            crc.step(word);
            let enc_word = enc.step(word, index);
            let check = self.spi.send32(delay, enc_word) >> 16;

            if check != (index & 0xFFFF) {
                return Err(MultibootError::TransmissionError);
            }
        }

        Ok(())
    }

    pub fn validate_checkum(
        &mut self,
        delay: &mut Delay,
        checksum: u16,
    ) -> Result<(), MultibootError> {
        while self.spi.send16(delay, 0x0065) != 0x0075 {}

        self.spi.send16(delay, 0x0066);
        let console_crc = self.spi.send16(delay, checksum);

        if console_crc == checksum {
            Ok(())
        } else {
            Err(MultibootError::InvalidChecksum)
        }
    }

    pub fn multiboot(&mut self, delay: &mut Delay) -> Result<(), MultibootError> {
        self.send_header(delay);
        let (mut crc, mut enc) = self.get_keys(delay)?;
        self.send_rom(delay, &mut crc, &mut enc)?;
        let checksum = crc.digest();
        self.validate_checkum(delay, checksum)
    }
}
