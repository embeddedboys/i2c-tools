#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use core::fmt::Write;

use ch32_hal::time::Hertz;
use ch32_hal::i2c::{Config, I2c};
use hal::println;
use embassy_executor::Spawner;
use embassy_time::{Delay, Duration, Timer};
use {ch32_hal as hal, panic_halt as _};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

#[embassy_executor::main(entry = "qingke_rt::entry")]
async fn main(spawner: Spawner) -> ! {
    hal::debug::SDIPrint::enable();
    let mut config = hal::Config::default();
    config.rcc = hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p = hal::init(config);
    hal::embassy::init();

    let scl = p.PB8;
    let sda = p.PB9;

    let i2c = I2c::new_blocking(p.I2C1, scl, sda, Hertz(400_000), Config::default());

    let iface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(iface, DisplaySize128x64, DisplayRotation::Rotate0).into_terminal_mode();
    display.init().unwrap();
    display.clear().unwrap();

    loop {
        for c in 97..123 {
            let _ = display.write_str(unsafe { core::str::from_utf8_unchecked(&[c]) });
        }
        for c in 65..91 {
            let _ = display.write_str(unsafe { core::str::from_utf8_unchecked(&[c]) });
        }
    }
}
