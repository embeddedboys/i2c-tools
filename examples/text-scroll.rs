#![no_std]
#![no_main]

use embassy_executor::Spawner;
use i2c_tools::{COLS, Fb, LedMatrix, ROWS};
use i2c_tools::font::{self, FONT_W, CHAR_SPACING};
use panic_halt as _;

fn build_text_fb(fb: &mut Fb, text: &[u8], offset: i32) {
    let char_step = (FONT_W + CHAR_SPACING) as i32;
    let total_width = text.len() as i32 * char_step;

    for row in fb.iter_mut() {
        for c in row.iter_mut() {
            *c = false;
        }
    }

    for scr_col in 0..COLS {
        let text_col = offset + scr_col as i32;
        if text_col < 0 || text_col >= total_width {
            continue;
        }
        let char_idx = (text_col / char_step) as usize;
        let glyph_col = (text_col % char_step) as usize;
        if char_idx >= text.len() || glyph_col >= FONT_W {
            continue;
        }
        let ch = text[char_idx] as char;
        let glyph = font::glyph(ch);
        let col_bits = glyph[glyph_col];
        for row in 0..ROWS {
            if col_bits & (1 << row) != 0 {
                fb[row][scr_col] = true;
            }
        }
    }
}

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(_spawner: Spawner) -> ! {
    let mut config = ch32_hal::Config::default();
    config.rcc = ch32_hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p = ch32_hal::init(config);

    let mut led = LedMatrix::new(p);

    let text = b"There it is. The Wishing Portal.\
They say for your wish to come true, \
you have to give up something really important. \
For me, that's my panini maker. \
I wish for a millon sandwiches. \
None of those things are gonna happen, you know. \
Morty wishes never come true. Not on the citadel. \
Then why did you bring us here? \
Because I wish that would change. \
I wish anything about this life would change. \
Well, I hope you're putting something pretty goddamn important in there. \
Me too. But i doubt it. \
!@#$%^&*()_+-=[]{}|\\;:',./<>?";

    let char_step = (FONT_W + CHAR_SPACING) as i32;
    let total_width = text.len() as i32 * char_step;
    let mut offset: i32 = -(COLS as i32);

    loop {
        let mut fb: Fb = [[false; COLS]; ROWS];
        build_text_fb(&mut fb, text, offset);
        led.scan(&fb, 10).await;

        offset += 1;
        if offset >= total_width + COLS as i32 {
            offset = -(COLS as i32);
        }
    }
}
