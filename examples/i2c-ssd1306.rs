#![no_std]
#![no_main]

use core::fmt::Write;

use ch32_hal::i2c::{Config, I2c};
use ch32_hal::time::Hertz;
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_halt as _;
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(_spawner: Spawner) -> ! {
    ch32_hal::debug::SDIPrint::enable();
    let mut config = ch32_hal::Config::default();
    config.rcc = ch32_hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p = ch32_hal::init(config);

    let scl = p.PB8;
    let sda = p.PB9;

    let i2c = I2c::new_blocking(p.I2C1, scl, sda, Hertz(400_000), Config::default());

    let iface = I2CDisplayInterface::new(i2c);
    let mut display =
        Ssd1306::new(iface, DisplaySize128x32, DisplayRotation::Rotate0).into_terminal_mode();
    display.init().unwrap();
    display.clear().unwrap();

    loop {
        for ch in 'a'..'z' {
            let _ = display.write_char(ch);
            Timer::after_millis(1).await;
        }
    }
}
