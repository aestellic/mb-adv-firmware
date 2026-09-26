use cortex_m::delay::Delay;
use embedded_hal::blocking::spi::Transfer;
use rp_pico::hal::{
    spi::{Enabled, SpiDevice},
    Spi,
};

pub trait GbaSpi: Transfer<u8> {
    fn send32(&mut self, delay: &mut Delay, data: u32) -> u32 {
        let buf = &mut data.to_be_bytes();
        let res = match self.transfer(buf) {
            Ok(recv) => u32::from_be_bytes(recv.try_into().unwrap()),
            Err(_) => 0,
        };

        delay.delay_us(10);
        res
    }

    fn send16(&mut self, delay: &mut Delay, data: u16) -> u16 {
        let recv = self.send32(delay, data as u32);
        (recv >> 16) as u16
    }
}

impl<D: SpiDevice> GbaSpi for Spi<Enabled, D, 8> {}
