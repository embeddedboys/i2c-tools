#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use ch32_hal::Peri;
use ch32_hal::gpio::{AnyPin, Level, Output, Speed};
use ch32_hal::i2c::{Config, I2c};
use ch32_hal::println;
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_halt as _;

// #[embassy_executor::task]
// async fn blink(p: ch32_hal::Peripherals, addr: u8, interval_ms: u64) {
//     let mut columns = [
//         Output::new(p.PB5, Level::Low, Speed::Low),
//         Output::new(p.PB4, Level::Low, Speed::Low),
//         Output::new(p.PB3, Level::Low, Speed::Low),
//         Output::new(p.PA15, Level::Low, Speed::Low),
//         Output::new(p.PA0, Level::Low, Speed::Low),
//         Output::new(p.PA1, Level::Low, Speed::Low),
//         Output::new(p.PA12, Level::Low, Speed::Low),
//         Output::new(p.PA11, Level::Low, Speed::Low),
//         Output::new(p.PA10, Level::Low, Speed::Low),
//         Output::new(p.PA9, Level::Low, Speed::Low),
//         Output::new(p.PA8, Level::Low, Speed::Low),
//         Output::new(p.PB15, Level::Low, Speed::Low),
//         Output::new(p.PB14, Level::Low, Speed::Low),
//         Output::new(p.PB13, Level::Low, Speed::Low),
//         Output::new(p.PB12, Level::Low, Speed::Low),
//         Output::new(p.PB11, Level::Low, Speed::Low),
//     ];

//     let mut rows = [
//         Output::new(p.PA3, Level::Low, Speed::Low),
//         Output::new(p.PA4, Level::Low, Speed::Low),
//         Output::new(p.PA5, Level::Low, Speed::Low),
//         Output::new(p.PA6, Level::Low, Speed::Low),
//         Output::new(p.PA7, Level::Low, Speed::Low),
//         Output::new(p.PB0, Level::Low, Speed::Low),
//         Output::new(p.PB1, Level::Low, Speed::Low),
//         Output::new(p.PB10, Level::Low, Speed::Low),
//     ];

//     loop {
//         for led in (0x0..0x7f).rev() {
//             let row = (led / 16) as usize;
//             let col = (led % 16) as usize;

//             rows[row].set_high();
//             columns[col].set_high();

//             Timer::after_millis(20).await;

//             rows[row].set_low();
//             columns[col].set_low();
//         }
//     }
// }

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(spawner: Spawner) -> ! {
    // ch32_hal::debug::SDIPrint::enable();
    // let p = ch32_hal::init(ch32_hal::Config::default());
    let mut config = ch32_hal::Config::default();
    config.rcc = ch32_hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p = ch32_hal::init(config);

    let scl = p.PB8;
    let sda = p.PB9;

    let mut i2c = I2c::new_blocking(
        p.I2C1,
        scl,
        sda,
        ch32_hal::time::Hertz(400_000),
        Config::default(),
    );

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
    rows.reverse();

    loop {
        let mut addr: u8 = 0;

        'outer: for i in (0..128).step_by(16) {
            for j in 0..16 {
                match i2c.blocking_write(i + j, &[0x00]) {
                    Ok(_) => {
                        addr = i + j;
                        break 'outer;
                    }
                    Err(_) => {}
                }
            }
        }

        let row = (addr / 16) as usize;
        let col = (addr % 16) as usize;

        // println!("will blink row {} col {}", row, col);
        rows[row].set_high();

        // for _ in 0..3 {
        columns[col].set_high();
        Timer::after_millis(500).await;
        columns[col].set_low();
        Timer::after_millis(500).await;
        // }
        rows[row].set_low();
    }
}
