#![no_std]
#![no_main]

use ch32_hal::gpio::{Level, Output, Speed};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_halt as _;

const COLS: usize = 16;
const ROWS: usize = 8;

/// Simple PRNG (xorshift32)
struct Rng(u32);

impl Rng {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
}

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

    // Each cell holds remaining lit scan cycles (0 = off)
    let mut stars: [[u8; COLS]; ROWS] = [[0; COLS]; ROWS];
    let mut rng = Rng(0xCAFE_BABE);

    loop {
        // Randomly light one new star
        {
            let r = (rng.next() as usize) % ROWS;
            let c = (rng.next() as usize) % COLS;
            if stars[r][c] == 0 {
                // Random duration: 20..80 scan cycles (~80..320ms)
                stars[r][c] = ((rng.next() % 60) + 20) as u8;
            }
        }

        // Scan the matrix once
        for row in 0..ROWS {
            for col in 0..COLS {
                if stars[row][col] > 0 {
                    columns[col].set_high();
                }
            }
            rows[row].set_high();
            Timer::after_micros(500).await;
            rows[row].set_low();
            for col in columns.iter_mut() {
                col.set_low();
            }
        }

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
