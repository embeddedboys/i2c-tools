#![no_std]
#![no_main]

use embassy_executor::Spawner;
use i2c_tools::{COLS, Fb, LedMatrix, Rng, ROWS};
use panic_halt as _;

/// Ground row (physical bottom, row 7)
const GROUND: usize = ROWS - 1;

/// Dino sprite: 2 columns wide, 4 rows tall
const DINO_COL: i8 = 2;
const DINO_W: i8 = 2;
const DINO_H: i8 = 4;

/// Cactus sprite: 1 column wide, 2 rows tall
const CACTUS_W: i8 = 1;
const CACTUS_H: i8 = 2;

/// How close the cactus must be before the dino jumps
const JUMP_DIST: i8 = 5;

/// Total jump arc length (up + down)
const JUMP_LEN: i8 = 10;

/// Game logic runs every TICK frames (higher = slower)
const TICK: u32 = 6;

struct Obstacle {
    col: i8,
}

impl Obstacle {
    fn new(col: i8) -> Self {
        Self { col }
    }
}

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(_spawner: Spawner) -> ! {
    let mut config = ch32_hal::Config::default();
    config.rcc = ch32_hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p = ch32_hal::init(config);

    let mut led = LedMatrix::new(p);
    let mut rng = Rng(0xD140_5EED);

    let mut obstacle = Obstacle::new(COLS as i8 - 1);
    let mut dino_row: i8 = 0; // 0 = ground, positive = height above ground
    let mut jumping = false;
    let mut jump_tick: i8 = 0;
    let mut frame: u32 = 0;
    let mut game_over = false;

    loop {
        if game_over {
            // Flash all LEDs briefly, then reset
            for _ in 0..50 {
                let fb: Fb = [[true; COLS]; ROWS];
                led.scan_once(&fb).await;
            }
            for _ in 0..50 {
                let fb: Fb = [[false; COLS]; ROWS];
                led.scan_once(&fb).await;
            }
            // Reset game
            obstacle = Obstacle::new(COLS as i8 - 1);
            dino_row = 0;
            jumping = false;
            jump_tick = 0;
            game_over = false;
            frame = 0;
            continue;
        }

        // --- Game logic every TICK frames ---
        if frame % TICK == 0 {
            // Scroll obstacle left
            obstacle.col -= 1;

            // Spawn new obstacle when it exits the screen
            if obstacle.col < -(CACTUS_W) {
                // Random distance: 8..15 columns ahead
                let gap = 8 + (rng.next() % 8) as i8;
                obstacle.col = COLS as i8 - 1 + gap;
            }

            // Auto-jump: when cactus is approaching and we're on the ground
            if !jumping
                && obstacle.col >= DINO_COL
                && obstacle.col - DINO_COL < JUMP_DIST
            {
                jumping = true;
                jump_tick = 0;
            }

            // Jump arc
            if jumping {
                jump_tick += 1;
                if jump_tick <= JUMP_LEN / 2 {
                    dino_row += 1;
                } else {
                    dino_row -= 1;
                }
                // Land back on ground
                if dino_row <= 0 {
                    dino_row = 0;
                    jumping = false;
                    jump_tick = 0;
                }
            }

            // Collision detection
            // row 0 = top, row 7 = bottom (GROUND)
            // Dino on ground: head at row (GROUND - DINO_H) = 3, feet at row (GROUND - 1) = 6
            // Jump by dino_row: subtract from row numbers (move toward row 0)
            let dino_head = (GROUND as i8) - DINO_H - dino_row;
            let dino_feet = (GROUND as i8) - 1 - dino_row;
            let cactus_top = (GROUND as i8) - CACTUS_H;
            let cactus_bot = (GROUND as i8) - 1;

            let col_overlap = obstacle.col < DINO_COL + DINO_W
                && obstacle.col + CACTUS_W > DINO_COL;
            let row_overlap = dino_head <= cactus_bot && dino_feet >= cactus_top;

            if col_overlap && row_overlap {
                game_over = true;
            }
        }

        // --- Render (row 0 = top, row 7 = bottom) ---
        let mut fb: Fb = [[false; COLS]; ROWS];

        // Ground line at row 7
        fb[GROUND] = [true; COLS];

        // Dino (2x4 block), head at row 3-dino_row, feet at row 6-dino_row
        let dino_top = (GROUND as i8) - DINO_H - dino_row;
        for dr in 0..DINO_H {
            let r = dino_top + dr;
            if r >= 0 && r < ROWS as i8 {
                for dc in 0..DINO_W {
                    let c = DINO_COL + dc;
                    if c >= 0 && c < COLS as i8 {
                        fb[r as usize][c as usize] = true;
                    }
                }
            }
        }
        // Dino "eye": clear top-right pixel (head row)
        if dino_top >= 0 && dino_top < ROWS as i8 && DINO_COL + 1 < COLS as i8 {
            fb[dino_top as usize][(DINO_COL + 1) as usize] = false;
        }

        // Cactus (1x2 block), rows 5..6
        let cactus_top = (GROUND as i8) - CACTUS_H;
        for dr in 0..CACTUS_H {
            let r = cactus_top + dr;
            if r >= 0 && r < ROWS as i8 {
                for dc in 0..CACTUS_W {
                    let c = obstacle.col + dc;
                    if c >= 0 && c < COLS as i8 {
                        fb[r as usize][c as usize] = true;
                    }
                }
            }
        }

        led.scan_once(&fb).await;
        frame = frame.wrapping_add(1);
    }
}
