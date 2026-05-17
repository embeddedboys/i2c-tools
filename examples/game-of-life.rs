#![no_std]
#![no_main]

use embassy_executor::Spawner;
use i2c_tools::{COLS, Fb, LedMatrix, Rng, ROWS};
use panic_halt as _;

/// Grid dimensions match the LED matrix
const W: usize = COLS;
const H: usize = ROWS;

/// Number of scan repeats per frame for persistence
const SCAN_REPEATS: u32 = 10;

/// Game of Life rules:
/// - Birth: exactly 3 live neighbors
/// - Survival: 2 or 3 live neighbors
/// - All else: die
const BIRTH: usize = 3;
const SURVIVE_MIN: usize = 2;
const SURVIVE_MAX: usize = 3;

/// Count live neighbors for cell (r, c), zero boundary
fn count_neighbors(grid: &[[bool; W]; H], r: usize, c: usize) -> usize {
    let mut count = 0;
    for dr in -1..=1 {
        for dc in -1..=1 {
            if dr == 0 && dc == 0 {
                continue;
            }
            let nr: i32 = r as i32 + dr;
            let nc: i32 = c as i32 + dc;
            if nr >= 0 && nr < H as i32 && nc >= 0 && nc < W as i32 {
                if grid[nr as usize][nc as usize] {
                    count += 1;
                }
            }
        }
    }
    count
}

/// Advance one generation (zero boundary)
fn next_gen(grid: &[[bool; W]; H]) -> [[bool; W]; H] {
    let mut next = [[false; W]; H];
    for r in 0..H {
        for c in 0..W {
            let n = count_neighbors(grid, r, c);
            next[r][c] = grid[r][c] && (n >= SURVIVE_MIN && n <= SURVIVE_MAX)
                || (!grid[r][c] && n == BIRTH);
        }
    }
    next
}

/// Check if all cells are dead
fn is_empty(grid: &[[bool; W]; H]) -> bool {
    for row in grid {
        for &cell in row {
            if cell {
                return false;
            }
        }
    }
    true
}

/// Seed the grid with a random pattern (~30% density)
fn random_seed(rng: &mut Rng) -> [[bool; W]; H] {
    let mut grid = [[false; W]; H];
    for r in 0..H {
        for c in 0..W {
            if (rng.next() % 100) < 30 {
                grid[r][c] = true;
            }
        }
    }
    grid
}

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(_spawner: Spawner) -> ! {
    let mut config = ch32_hal::Config::default();
    config.rcc = ch32_hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p = ch32_hal::init(config);

    let mut led = LedMatrix::new(p);
    let mut rng = Rng(0xCAFE_BABE);

    let mut grid: [[bool; W]; H] = random_seed(&mut rng);
    let mut frame: u32 = 0;
    const GENERATION_INTERVAL: u32 = 8;

    loop {
        if frame % GENERATION_INTERVAL == 0 {
            grid = next_gen(&grid);

            // Reset if all cells died
            if is_empty(&grid) {
                grid = random_seed(&mut rng);
            }
        }

        // Build framebuffer
        let mut fb: Fb = [[false; W]; H];
        for r in 0..H {
            for c in 0..W {
                fb[r][c] = grid[r][c];
            }
        }

        led.scan(&fb, SCAN_REPEATS).await;
        frame += 1;
    }
}
