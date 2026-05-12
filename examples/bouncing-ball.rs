#![no_std]
#![no_main]

use embassy_executor::Spawner;
use i2c_tools::{COLS, Fb, LedMatrix, ROWS};
use panic_halt as _;

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(_spawner: Spawner) -> ! {
    let mut config = ch32_hal::Config::default();
    config.rcc = ch32_hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p = ch32_hal::init(config);

    let mut led = LedMatrix::new(p);

    // Ball position and velocity (fixed-point 8.8)
    let mut x: i16 = 3 << 8;
    let mut y: i16 = 2 << 8;
    let mut vx: i16 = 180;
    let mut vy: i16 = 130;

    let max_x: i16 = ((COLS - 1) as i16) << 8;
    let max_y: i16 = ((ROWS - 1) as i16) << 8;

    loop {
        x += vx;
        y += vy;

        if x <= 0 {
            x = 0;
            vx = -vx;
        }
        if x >= max_x {
            x = max_x;
            vx = -vx;
        }
        if y <= 0 {
            y = 0;
            vy = -vy;
        }
        if y >= max_y {
            y = max_y;
            vy = -vy;
        }

        let col = (x >> 8) as usize;
        let row = (y >> 8) as usize;

        let mut fb: Fb = [[false; COLS]; ROWS];
        fb[row][col] = true;
        led.scan(&fb, 20).await;
    }
}
