#![no_std]
#![no_main]

mod gba;
use cortex_m::delay::Delay;
use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;
use fugit::RateExtU32;
use gba::MultibootError;
use panic_halt as _;
use rp_pico::entry;
use rp_pico::hal;
use rp_pico::hal::gpio::bank0::Gpio25;
use rp_pico::hal::gpio::{Output, Pin, PushPull};
use rp_pico::hal::{gpio, spi};
use rp_pico::hal::{pac, Clock};

type LedPin = Pin<Gpio25, Output<PushPull>>;

fn blink(led_pin: &mut LedPin, delay: &mut Delay, count: usize) {
    for _ in 0..count {
        led_pin.set_high().unwrap();
        delay.delay_ms(300);
        led_pin.set_low().unwrap();
        delay.delay_ms(300);
    }
}

const MB_ROM_1: &[u8] = include_bytes!("../mb_1.gba");
const MB_ROM_2: &[u8] = include_bytes!("../mb_2.gba");
const MB_ROM_3: &[u8] = include_bytes!("../mb_3.gba");

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let sio = hal::Sio::new(pac.SIO);
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.led.into_push_pull_output();

    //blink(&mut led_pin, &mut delay, 1);

    let opt1 = pins.gpio17.into_pull_up_input();
    let opt2 = pins.gpio21.into_pull_up_input();
    let opt3 = pins.gpio22.into_pull_up_input();

    // These are implicitly used by the spi driver if they are in the correct mode
    let _spi_mosi = pins.gpio20.into_mode::<gpio::FunctionSpi>();
    let _spi_sclk = pins.gpio6.into_mode::<gpio::FunctionSpi>();
    let _spi_miso = pins.gpio7.into_mode::<gpio::FunctionSpi>();

    // Create an SPI driver instance for the SPI0 device
    let spi = spi::Spi::new(pac.SPI0);

    // Exchange the uninitialised SPI driver for an initialised one
    let mut spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        256_000u32.Hz(),
        &embedded_hal::spi::MODE_3,
    );

    let mut gba = gba::Gba::new(&mut spi, MB_ROM_1);

    let mut has_multibooted = false;
    let mut opt1_selected = false;
    let mut opt2_selected = false;
    let mut opt3_selected = false;

    loop {
        if gba.is_ready(&mut delay) && !has_multibooted {
            has_multibooted = true;

            match gba.multiboot(&mut delay) {
                Err(MultibootError::FailedHandshake) => {
                    blink(&mut led_pin, &mut delay, 2);
                }
                Err(MultibootError::InvalidChecksum) => {
                    blink(&mut led_pin, &mut delay, 3);
                }
                Err(MultibootError::TransmissionError) => {
                    blink(&mut led_pin, &mut delay, 4);
                }
                Ok(_) => {
                    blink(&mut led_pin, &mut delay, 1);
                }
            };
        }
        else {
            if opt1.is_low().unwrap() && !opt1_selected {
                opt1_selected = true;
                opt2_selected = false;
                opt3_selected = false;
                gba = gba::Gba::new(&mut spi, MB_ROM_1);
                blink(&mut led_pin, &mut delay, 1);
            }
            else if opt2.is_low().unwrap() && !opt2_selected {
                opt1_selected = false;
                opt2_selected = true;
                opt3_selected = false;
                gba = gba::Gba::new(&mut spi, MB_ROM_2);
                blink(&mut led_pin, &mut delay, 2);
            }
            else if opt3.is_low().unwrap() && !opt3_selected {
                opt1_selected = false;
                opt2_selected = false;
                opt3_selected = true;                
                gba = gba::Gba::new(&mut spi, MB_ROM_3);
                blink(&mut led_pin, &mut delay, 3);
            }
        }
    }
}
