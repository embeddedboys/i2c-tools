#![no_std]
#![no_main]

use embassy_executor::Spawner;
use i2c_tools::{COLS, Fb, LedMatrix, Rng, ROWS};
use panic_halt as _;

/// Tetris piece shapes (4 rotations × 4x4 bitmap)
/// Each u16 encodes a 4×4 grid, LSB = top-left
const PIECES: [[u16; 4]; 7] = [
    // I
    [0x00F0, 0x2222, 0x00F0, 0x2222],
    // O
    [0x0660, 0x0660, 0x0660, 0x0660],
    // T
    [0x0E40, 0x4C40, 0x04E0, 0x4640],
    // S
    [0x06C0, 0x8C40, 0x06C0, 0x8C40],
    // Z
    [0x0C60, 0x4C80, 0x0C60, 0x4C80],
    // L
    [0x0E80, 0xC440, 0x2E00, 0x44C0],
    // J
    [0x0E20, 0x44C0, 0x8E00, 0xC880],
];

/// Get cell (r, c) from a piece bitmap (4×4)
fn piece_cell(shape: u16, r: usize, c: usize) -> bool {
    (shape >> (r * 4 + c)) & 1 != 0
}

/// Board: true = occupied
struct Board {
    cells: [[bool; COLS]; ROWS],
}

impl Board {
    fn new() -> Self {
        Self { cells: [[false; COLS]; ROWS] }
    }

    /// Check if piece fits at (col, row) with given shape
    fn fits(&self, shape: u16, col: i8, row: i8) -> bool {
        for r in 0..4 {
            for c in 0..4 {
                if !piece_cell(shape, r, c) {
                    continue;
                }
                let br = row + r as i8;
                let bc = col + c as i8;
                if bc < 0 || bc >= COLS as i8 || br >= ROWS as i8 {
                    return false;
                }
                if br >= 0 && self.cells[br as usize][bc as usize] {
                    return false;
                }
            }
        }
        true
    }

    /// Lock piece into board
    fn lock(&mut self, shape: u16, col: i8, row: i8) {
        for r in 0..4 {
            for c in 0..4 {
                if !piece_cell(shape, r, c) {
                    continue;
                }
                let br = row + r as i8;
                let bc = col + c as i8;
                if br >= 0 && br < ROWS as i8 && bc >= 0 && bc < COLS as i8 {
                    self.cells[br as usize][bc as usize] = true;
                }
            }
        }
    }

    /// Clear full lines, return count
    fn clear_lines(&mut self) -> usize {
        let mut cleared = 0;
        let mut write = ROWS - 1;
        for r in (0..ROWS).rev() {
            if self.cells[r].iter().all(|&c| c) {
                cleared += 1;
            } else {
                if write != r {
                    self.cells[write] = self.cells[r];
                }
                if write > 0 {
                    write -= 1;
                }
            }
        }
        // Clear top rows
        for r in 0..cleared {
            self.cells[r] = [false; COLS];
        }
        cleared
    }

    /// Count gaps (holes) in the board
    fn holes(&self) -> usize {
        let mut count = 0;
        for col in 0..COLS {
            let mut blocked = false;
            for row in 0..ROWS {
                if self.cells[row][col] {
                    blocked = true;
                } else if blocked {
                    count += 1;
                }
            }
        }
        count
    }

    /// Height of tallest column
    fn max_height(&self) -> usize {
        for row in 0..ROWS {
            if self.cells[row].iter().any(|&c| c) {
                return ROWS - row;
            }
        }
        0
    }

}

/// Find best placement for a piece using greedy scoring.
/// Returns (col, rotation).
fn best_move(board: &Board, piece_idx: usize) -> (i8, usize) {
    let mut best_col = 0i8;
    let mut best_rot = 0usize;
    let mut best_score = i32::MIN;

    for rot in 0..4 {
        let shape = PIECES[piece_idx][rot];
        // Find valid column range
        let min_c = -3i8;
        let max_c = COLS as i8;
        for col in min_c..max_c {
            if !board.fits(shape, col, 0) {
                continue;
            }
            // Drop to lowest position
            let mut drop_row = 0i8;
            while board.fits(shape, col, drop_row + 1) {
                drop_row += 1;
            }

            // Score: lower is better, fewer holes is better, clear lines is best
            let mut test_board = Board { cells: board.cells };
            test_board.lock(shape, col, drop_row);
            let lines = test_board.clear_lines();
            let holes = test_board.holes() as i32;
            let height = test_board.max_height() as i32;

            let score = lines as i32 * 100 - holes * 10 - height * 2 + drop_row as i32;
            if score > best_score {
                best_score = score;
                best_col = col;
                best_rot = rot;
            }
        }
    }
    (best_col, best_rot)
}

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(_spawner: Spawner) -> ! {
    let mut config = ch32_hal::Config::default();
    config.rcc = ch32_hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p = ch32_hal::init(config);

    let mut led = LedMatrix::new(p);
    let mut rng = Rng(0x5E77_15);

    let mut board = Board::new();
    let mut piece_idx = (rng.next() % 7) as usize;
    let mut frame: u32 = 0;

    // Auto-play: compute best move, then animate the drop
    let (mut target_col, mut target_rot) = best_move(&board, piece_idx);
    let mut cur_col = COLS as i8 / 2 - 2;
    let mut cur_rot = 0usize;
    let mut cur_row = 0i8;

    loop {
        // Move toward target: rotate and shift (no drop until in position)
        if frame % 2 == 0 {
            let at_target = cur_col == target_col && cur_rot == target_rot;

            if !at_target {
                // Rotate toward target
                if cur_rot != target_rot {
                    let next_rot = (cur_rot + 1) % 4;
                    if board.fits(PIECES[piece_idx][next_rot], cur_col, cur_row) {
                        cur_rot = next_rot;
                    }
                }

                // Move toward target column
                if cur_col < target_col {
                    if board.fits(PIECES[piece_idx][cur_rot], cur_col + 1, cur_row) {
                        cur_col += 1;
                    }
                } else if cur_col > target_col {
                    if board.fits(PIECES[piece_idx][cur_rot], cur_col - 1, cur_row) {
                        cur_col -= 1;
                    }
                }
            }
        }

        // Drop every 6 frames (always, not just at target)
        if frame % 6 == 0 {
            let shape = PIECES[piece_idx][cur_rot];
            if board.fits(shape, cur_col, cur_row + 1) {
                cur_row += 1;
            } else {
                // Lock piece
                board.lock(shape, cur_col, cur_row);
                board.clear_lines();

                // Spawn next piece
                piece_idx = (rng.next() % 7) as usize;
                let (tc, tr) = best_move(&board, piece_idx);
                target_col = tc;
                target_rot = tr;
                cur_col = COLS as i8 / 2 - 2;
                cur_rot = 0;
                cur_row = 0i8;

                // Check game over
                if !board.fits(PIECES[piece_idx][cur_rot], cur_col, cur_row) {
                    // Reset
                    board = Board::new();
                }
            }
        }

        // Render: board + falling piece (flip vertically for display)
        let mut fb: Fb = [[false; COLS]; ROWS];
        let shape = PIECES[piece_idx][cur_rot];
        for r in 0..ROWS {
            for c in 0..COLS {
                if board.cells[r][c] {
                    fb[ROWS - 1 - r][c] = true;
                }
            }
        }
        for r in 0..4 {
            for c in 0..4 {
                if piece_cell(shape, r, c) {
                    let br = cur_row + r as i8;
                    let bc = cur_col + c as i8;
                    if br >= 0 && br < ROWS as i8 && bc >= 0 && bc < COLS as i8 {
                        fb[ROWS - 1 - br as usize][bc as usize] = true;
                    }
                }
            }
        }

        led.scan(&fb, 10).await;
        frame += 1;
    }
}
