#![no_std]
#![no_main]

use embassy_executor::Spawner;
use i2c_tools::{COLS, Fb, LedMatrix, Rng, ROWS};
use panic_halt as _;

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(_spawner: Spawner) -> ! {
    let mut config = ch32_hal::Config::default();
    config.rcc = ch32_hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p = ch32_hal::init(config);

    let mut led = LedMatrix::new(p);

    let mut stars: [[u8; COLS]; ROWS] = [[0; COLS]; ROWS];
    let mut rng = Rng(0xCAFE_BABE);

    loop {
        // Randomly light one new star
        let r = (rng.next() as usize) % ROWS;
        let c = (rng.next() as usize) % COLS;
        if stars[r][c] == 0 {
            stars[r][c] = ((rng.next() % 60) + 20) as u8;
        }

        // Build framebuffer from star timers
        let mut fb: Fb = [[false; COLS]; ROWS];
        for row in 0..ROWS {
            for col in 0..COLS {
                fb[row][col] = stars[row][col] > 0;
            }
        }
        led.scan_once(&fb).await;

        // Decay lit stars
        for row in 0..ROWS {
            for col in 0..COLS {
                if stars[row][col] > 0 {
                    stars[row][col] -= 1;
                }
            }
        }
    }
}
