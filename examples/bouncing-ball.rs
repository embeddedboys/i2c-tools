#![no_std]
#![no_main]

use ch32_hal::gpio::{Level, Output, Speed};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_halt as _;

const COLS: usize = 16;
const ROWS: usize = 8;

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(_spawner: Spawner) -> ! {
    let mut config = ch32_hal::Config::default();
    config.rcc = ch32_hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p = ch32_hal::init(config);

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

    // Ball position and velocity (fixed-point 8.8)
    let mut x: i16 = 3 << 8;
    let mut y: i16 = 2 << 8;
    let mut vx: i16 = 180;
    let mut vy: i16 = 130;

    let max_x: i16 = ((COLS - 1) as i16) << 8;
    let max_y: i16 = ((ROWS - 1) as i16) << 8;

    loop {
        // Move
        x += vx;
        y += vy;

        // Bounce off walls
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

        // Determine which LED to light
        let col = (x >> 8) as usize;
        let row = (y >> 8) as usize;

        // Scan the matrix — only one LED is on per frame
        for r in 0..ROWS {
            if r == row {
                columns[col].set_high();
            }
            rows[r].set_high();
            Timer::after_micros(1000).await;
            rows[r].set_low();
            columns[col].set_low();
        }

        // Control speed
        Timer::after_millis(100).await;
    }
}
