#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use i2c_tools::{COLS, Fb, LedMatrix, ROWS};
use panic_halt as _;

/// Maximum length the snake can grow to
const MAX_LEN: usize = 128;
/// Number of frames between each snake move
const MOVE_INTERVAL: u32 = 18;
/// Total grid size = columns * rows
const GRID_SIZE: usize = COLS * ROWS;

/// Represents a position on the LED matrix grid
#[derive(Copy, Clone, PartialEq)]
struct Pos {
    x: i8,
    y: i8,
}

impl Pos {
    /// Calculate a new position by stepping in the given direction
    fn step(self, dx: i8, dy: i8) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

/// Represents the snake game state
struct Snake {
    /// Array storing all body segment positions (head at index 0)
    body: [Pos; MAX_LEN],
    /// Current length of the snake
    len: usize,
    /// Current movement direction (x component)
    dx: i8,
    /// Current movement direction (y component)
    dy: i8,
}

impl Snake {
    /// Create a new snake with initial position and direction
    fn new() -> Self {
        let mut body = [Pos { x: 0, y: 0 }; MAX_LEN];
        // Start at top-left corner, facing right
        body[0] = Pos { x: 0, y: 0 };
        Self {
            body,
            len: 1,
            dx: 1,
            dy: 0,
        }
    }

    /// Get the head position of the snake
    fn head(&self) -> Pos {
        self.body[0]
    }

    /// Move snake forward one step in current direction
    fn step(&mut self) {
        for i in (1..self.len).rev() {
            self.body[i] = self.body[i - 1];
        }
        self.body[0] = self.body[0].step(self.dx, self.dy);
    }

    /// Grow snake by one segment
    fn grow(&mut self) {
        if self.len < MAX_LEN {
            self.len += 1;
        }
    }
}

/// Predefined path to fill the entire grid row by row
/// Returns the next position in the path, or None if grid is full
fn get_next_target(current_target: Pos) -> Option<Pos> {
    let x = current_target.x;
    let y = current_target.y;
    
    if x < (COLS - 1) as i8 {
        // Continue right on current row
        Some(Pos { x: x + 1, y })
    } else if y < (ROWS - 1) as i8 {
        // At end of row, go down to next row
        Some(Pos { x: 0, y: y + 1 })
    } else {
        // Filled entire grid!
        None
    }
}

/// Get direction from one position to adjacent position
fn get_direction(from: Pos, to: Pos) -> (i8, i8) {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    
    (dx, dy)
}

/// Main game loop
#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(_spawner: Spawner) -> ! {
    // Initialize hardware and clock configuration
    let mut config = ch32_hal::Config::default();
    config.rcc = ch32_hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p = ch32_hal::init(config);

    // Initialize LED matrix driver
    let mut led = LedMatrix::new(p);

    // Initialize game state
    let mut snake = Snake::new();
    let mut target = Pos { x: 0, y: 0 };
    let mut frame: u32 = 0;
    let mut is_full = false;

    // Main game loop
    loop {
        if is_full {
            // Filled entire grid! Blink all LEDs
            let mut fb: Fb = [[true; COLS]; ROWS];
            led.scan_once(&fb).await;
            Timer::after_millis(100).await;
            fb = [[false; COLS]; ROWS];
            led.scan_once(&fb).await;
            Timer::after_millis(100).await;
            continue;
        }

        // Update game logic at fixed intervals
        if frame % MOVE_INTERVAL == 0 {
            // If we reached current target, grow and get next target
            if snake.head() == target {
                snake.grow();
                if let Some(next_target) = get_next_target(target) {
                    target = next_target;
                } else {
                    // Filled entire grid!
                    is_full = true;
                    frame = frame.wrapping_add(1);
                    continue;
                }
            }

            // Move towards target
            let (dx, dy) = get_direction(snake.head(), target);
            snake.dx = dx;
            snake.dy = dy;
            snake.step();
        }

        // Render frame - draw snake and target on LED matrix
        let mut fb: Fb = [[false; COLS]; ROWS];
        for i in 0..snake.len {
            fb[snake.body[i].y as usize][snake.body[i].x as usize] = true;
        }
        if !is_full {
            fb[target.y as usize][target.x as usize] = true;
        }

        // Scan LED matrix to display the frame
        led.scan_once(&fb).await;
        frame = frame.wrapping_add(1);
    }
}
