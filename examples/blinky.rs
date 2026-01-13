#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use ch32_hal::Peri;
use ch32_hal::gpio::{AnyPin, Level, Output, Speed};
use ch32_hal::println;
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_halt as _;

#[embassy_executor::task]
async fn blink(p: ch32_hal::Peripherals) {
    let mut columns = [
        Output::new(p.PB5, Level::Low, Speed::Low),
        Output::new(p.PB4, Level::Low, Speed::Low),
        Output::new(p.PB3, Level::Low, Speed::Low),
        Output::new(p.PA15, Level::Low, Speed::Low),
        Output::new(p.PA0, Level::Low, Speed::Low),
        Output::new(p.PA1, Level::Low, Speed::Low),
        Output::new(p.PA12, Level::Low, Speed::Low),
        Output::new(p.PA11, Level::Low, Speed::Low),
        Output::new(p.PA10, Level::Low, Speed::Low),
        Output::new(p.PA9, Level::Low, Speed::Low),
        Output::new(p.PA8, Level::Low, Speed::Low),
        Output::new(p.PB15, Level::Low, Speed::Low),
        Output::new(p.PB14, Level::Low, Speed::Low),
        Output::new(p.PB13, Level::Low, Speed::Low),
        Output::new(p.PB12, Level::Low, Speed::Low),
        Output::new(p.PB11, Level::Low, Speed::Low),
    ];

    let mut rows = [
        Output::new(p.PA3, Level::Low, Speed::Low),
        Output::new(p.PA4, Level::Low, Speed::Low),
        Output::new(p.PA5, Level::Low, Speed::Low),
        Output::new(p.PA6, Level::Low, Speed::Low),
        Output::new(p.PA7, Level::Low, Speed::Low),
        Output::new(p.PB0, Level::Low, Speed::Low),
        Output::new(p.PB1, Level::Low, Speed::Low),
        Output::new(p.PB10, Level::Low, Speed::Low),
    ];

    loop {
        for led in (0x0..0x7f).rev() {
            let row = (led / 16) as usize;
            let col = (led % 16) as usize;

            rows[row].set_high();
            columns[col].set_high();

            Timer::after_millis(20).await;

            rows[row].set_low();
            columns[col].set_low();
        }
    }
}

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(spawner: Spawner) -> ! {
    // ch32_hal::debug::SDIPrint::enable();
    let mut config = ch32_hal::Config::default();
    config.rcc = ch32_hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p = ch32_hal::init(config);

    // Adjust the LED GPIO according to your board
    spawner.spawn(blink(p)).unwrap();
    loop {
        Timer::after_millis(1000).await;
        // println!("tick");
    }
}
