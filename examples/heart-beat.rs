#![no_std]
#![no_main]

use embassy_executor::Spawner;
use i2c_tools::{COLS, Fb, LedMatrix, ROWS};
use panic_halt as _;

/// Build a small heart (~6 wide × 4 tall) centred on the 8×16 grid.
fn small_heart() -> Fb {
    let mut fb = [[false; COLS]; ROWS];
    // Row 2: two bumps
    fb[2][5] = true;
    fb[2][6] = true;
    fb[2][9] = true;
    fb[2][10] = true;
    // Row 3: wide band
    for c in 4..12 {
        fb[3][c] = true;
    }
    // Row 4: wide band
    for c in 4..12 {
        fb[4][c] = true;
    }
    // Row 5: bottom point
    for c in 5..11 {
        fb[5][c] = true;
    }
    fb
}

/// Build a medium heart (~10 wide × 6 tall).
fn medium_heart() -> Fb {
    let mut fb = [[false; COLS]; ROWS];
    // Row 1: two bumps
    fb[1][3] = true;
    fb[1][4] = true;
    fb[1][11] = true;
    fb[1][12] = true;
    // Row 2
    for c in 2..7 {
        fb[2][c] = true;
    }
    for c in 9..14 {
        fb[2][c] = true;
    }
    // Row 3–4: full wide body
    for r in 3..=4 {
        for c in 1..15 {
            fb[r][c] = true;
        }
    }
    // Row 5: taper
    for c in 2..14 {
        fb[5][c] = true;
    }
    // Row 6: point
    for c in 3..13 {
        fb[6][c] = true;
    }
    fb
}

/// Build a large heart (~14 wide × 7 tall).
fn large_heart() -> Fb {
    let mut fb = [[false; COLS]; ROWS];
    // Row 0: top bumps
    fb[0][2] = true;
    fb[0][3] = true;
    fb[0][12] = true;
    fb[0][13] = true;
    // Row 1
    for c in 1..6 {
        fb[1][c] = true;
    }
    for c in 10..15 {
        fb[1][c] = true;
    }
    // Row 2
    for c in 0..7 {
        fb[2][c] = true;
    }
    for c in 9..16 {
        fb[2][c] = true;
    }
    // Row 3–4: widest
    for r in 3..=4 {
        for c in 0..16 {
            fb[r][c] = true;
        }
    }
    // Row 5: taper
    for c in 1..15 {
        fb[5][c] = true;
    }
    // Row 6: point
    for c in 2..14 {
        fb[6][c] = true;
    }
    // Row 7: tip
    for c in 4..12 {
        fb[7][c] = true;
    }
    fb
}

/// Heart-beat phase: which heart to show + how many frames.
struct Phase {
    heart: Fb,
    frames: u32,
}

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(_spawner: Spawner) -> ! {
    let mut config = ch32_hal::Config::default();
    config.rcc = ch32_hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p = ch32_hal::init(config);

    let mut led = LedMatrix::new(p);

    // Heart-beat cycle: diastole → rapid expansion → peak → rapid contraction
    // ~72 BPM with scan(&fb, 4) at ~21 ms/frame.
    let phases = [
        Phase {
            heart: small_heart(),
            frames: 18, // diastole (relaxed)
        },
        Phase {
            heart: medium_heart(),
            frames: 4, // expanding
        },
        Phase {
            heart: large_heart(),
            frames: 4, // peak contraction
        },
        Phase {
            heart: medium_heart(),
            frames: 4, // contracting
        },
    ];

    let mut phase_idx: usize = 0;
    let mut frame_in_phase: u32 = 0;

    loop {
        let phase = &phases[phase_idx];
        led.scan(&phase.heart, 4).await;

        frame_in_phase += 1;
        if frame_in_phase >= phase.frames {
            frame_in_phase = 0;
            phase_idx = (phase_idx + 1) % phases.len();
        }
    }
}
