#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use i2c_tools::{COLS, LedMatrix, ROWS};
use panic_halt as _;

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(_spawner: Spawner) -> ! {
    let mut config = ch32_hal::Config::default();
    config.rcc = ch32_hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p = ch32_hal::init(config);

    let mut led = LedMatrix::new(p);

    loop {
        for led_idx in 0..(COLS * ROWS) {
            let row = led_idx / COLS;
            let col = led_idx % COLS;

            let mut fb = [[false; COLS]; ROWS];
            fb[row][col] = true;
            led.scan(&fb, 10).await;

            Timer::after_millis(20).await;
        }
    }
}
