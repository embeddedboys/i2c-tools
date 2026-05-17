#![no_std]
#![no_main]

use embassy_executor::Spawner;
use i2c_tools::{COLS, Fb, LedMatrix, Rng, ROWS};
use panic_halt as _;

/// Ball speed (cells per update)
const BALL_SPEED_X: i8 = 1;
const BALL_SPEED_Y: i8 = 1;
/// Paddle speed limit (cells per update)
const PADDLE_SPEED: i8 = 1;
/// Paddle width in cells
const PADDLE_W: usize = 4;
/// Frames between ball updates
const MOVE_INTERVAL: u32 = 8;
/// Brick grid dimensions
const BRICK_ROWS: usize = 4;
const BRICK_COLS: usize = 7;

/// Ball state
struct Ball {
    x: i8,
    y: i8,
    dx: i8,
    dy: i8,
}

impl Ball {
    fn new() -> Self {
        Self {
            x: COLS as i8 / 2,
            y: ROWS as i8 / 2,
            dx: BALL_SPEED_X,
            dy: -BALL_SPEED_Y, // start moving up
        }
    }

    /// Reset ball to center, launch toward bottom
    fn reset(&mut self, rng: &mut Rng) {
        self.x = COLS as i8 / 2;
        self.y = ROWS as i8 / 2 + 1;
        self.dx = if (rng.next() % 2) == 0 { -BALL_SPEED_X } else { BALL_SPEED_X };
        self.dy = BALL_SPEED_Y; // always launch downward
    }

    /// Move ball one step, clamping to grid bounds
    fn step(&mut self) {
        self.x += self.dx;
        self.y += self.dy;

        // Clamp and bounce off left/right walls
        if self.x < 0 {
            self.x = 0;
            self.dx = -self.dx;
        } else if self.x >= COLS as i8 {
            self.x = COLS as i8 - 1;
            self.dx = -self.dx;
        }

        // Bounce off top
        if self.y < 0 {
            self.y = 0;
            self.dy = -self.dy;
        }
    }
}

/// Paddle: horizontal bar at the bottom
struct Paddle {
    x: i8, // left edge
}

impl Paddle {
    fn new() -> Self {
        Self {
            x: ((COLS - PADDLE_W) / 2) as i8,
        }
    }

    /// Move paddle toward target X, limited by PADDLE_SPEED
    fn move_toward(&mut self, target: i8) {
        let max_x = (COLS - PADDLE_W) as i8;
        let target = target.clamp(0, max_x);
        let diff = target - self.x;
        let abs_diff = diff.abs();
        if abs_diff > PADDLE_SPEED {
            self.x += diff.signum() * PADDLE_SPEED;
        } else {
            self.x = target;
        }
    }

    /// Check if paddle covers column cx
    fn covers(&self, cx: i8) -> bool {
        cx >= self.x && cx < self.x + PADDLE_W as i8
    }
}

/// Brick grid
struct Bricks {
    /// [row][col] = alive
    grid: [[bool; BRICK_COLS]; BRICK_ROWS],
}

impl Bricks {
    fn new() -> Self {
        let mut grid = [[false; BRICK_COLS]; BRICK_ROWS];
        for r in 0..BRICK_ROWS {
            for c in 0..BRICK_COLS {
                grid[r][c] = true;
            }
        }
        Self { grid }
    }

    /// Check if pixel position (col, row) overlaps a brick.
    /// If so, break it and return true.
    fn hit_brick(&mut self, px: i8, py: i8) -> bool {
        if px < 0 || py < 0 {
            return false;
        }
        let px = px as usize;
        let py = py as usize;
        if py >= BRICK_ROWS {
            return false;
        }
        if px >= 1 {
            let gc = (px - 1) / 2;
            if gc < BRICK_COLS && self.grid[py][gc] {
                self.grid[py][gc] = false;
                return true;
            }
        }
        false
    }

    /// Check if all bricks are destroyed
    fn is_empty(&self) -> bool {
        for row in self.grid {
            for &alive in row.iter() {
                if alive {
                    return false;
                }
            }
        }
        true
    }
}

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(_spawner: Spawner) -> ! {
    let mut config = ch32_hal::Config::default();
    config.rcc = ch32_hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p = ch32_hal::init(config);

    let mut led = LedMatrix::new(p);
    let mut rng = Rng(0xDEAD_BEEF);

    let mut ball = Ball::new();
    let mut paddle = Paddle::new();
    let mut bricks = Bricks::new();

    // State: 0 = playing, 1 = ball lost, 2 = won
    let mut state: u8 = 0;
    let mut state_timer: u32 = 0;
    const STATE_DELAY: u32 = 90;

    let mut frame: u32 = 0;

    loop {
        if state == 0 {
            // Update paddle to track ball
            if ball.dx > 0 || ball.dy > 0 {
                let target = ball.x - PADDLE_W as i8 / 2;
                paddle.move_toward(target);
            }

            // Update ball at fixed interval
            if frame % MOVE_INTERVAL == 0 {
                // Brick collision: check cells along the ball's path BEFORE moving.
                // Ball moves 1 cell at a time, so check the path from (x, y) to (next_x, next_y).
                let next_x = ball.x + ball.dx;
                let next_y = ball.y + ball.dy;

                let mut hit_brick = false;

                // Check all cells along the path (including start and end)
                let (mut cx, mut cy) = (ball.x, ball.y);
                let (dx, dy) = (ball.dx, ball.dy);
                loop {
                    if cx >= 0 && cx < COLS as i8 && cy >= 0 && cy < ROWS as i8 {
                        if bricks.hit_brick(cx, cy) {
                            hit_brick = true;
                            if dy > 0 {
                                // Moving down — bounce up
                                ball.dy = -BALL_SPEED_Y;
                            } else if dy < 0 {
                                // Moving up — bounce down
                                ball.dy = BALL_SPEED_Y;
                            } else {
                                ball.dy = BALL_SPEED_Y;
                            }
                            if dx > 0 {
                                ball.dx = BALL_SPEED_X;
                            } else if dx < 0 {
                                ball.dx = -BALL_SPEED_X;
                            }
                            ball.dx += if (rng.next() % 2) == 0 { -1i8 } else { 1i8 };
                            if ball.dx > BALL_SPEED_X { ball.dx = BALL_SPEED_X; }
                            if ball.dx < -BALL_SPEED_X { ball.dx = -BALL_SPEED_X; }
                            break;
                        }
                    }
                    // Step along path
                    if cx == next_x && cy == next_y {
                        break;
                    }
                    if cx < next_x { cx += 1; } else if cx > next_x { cx -= 1; }
                    if cy < next_y { cy += 1; } else if cy > next_y { cy -= 1; }
                }

                // Only move if no brick hit
                if !hit_brick {
                    ball.step();
                }

                // Check ball fell off bottom
                if ball.y >= ROWS as i8 {
                    state = 1;
                    state_timer = STATE_DELAY;
                }

                // Paddle collision (ball must be just above paddle row, moving down)
                if ball.dy > 0
                    && ball.y >= (ROWS - 1) as i8
                    && ball.x >= 0
                    && ball.x < COLS as i8
                    && paddle.covers(ball.x)
                {
                    ball.y = (ROWS - 1) as i8 - 1; // push above paddle
                    ball.dy = -ball.dy;

                    // Angle based on hit position relative to paddle center
                    let center = paddle.x + PADDLE_W as i8 / 2;
                    let offset = ball.x - center;
                    ball.dx = offset;
                    // Ensure minimum horizontal movement
                    if ball.dx == 0 {
                        ball.dx = 1;
                    }
                }

                // Check win condition
                if bricks.is_empty() {
                    state = 2;
                    state_timer = STATE_DELAY;
                }
            }
        } else if state == 1 {
            // Ball lost — countdown
            state_timer -= 1;
            if state_timer == 0 {
                ball.reset(&mut rng);
                state = 0;
            }
        } else {
            // Won — just show the empty board with ball
            state_timer -= 1;
            if state_timer == 0 {
                // Restart with new random ball direction
                ball.reset(&mut rng);
                bricks = Bricks::new();
                state = 0;
            }
        }

        // Build framebuffer
        let mut fb: Fb = [[false; COLS]; ROWS];

        // Draw ball
        if ball.x >= 0 && ball.x < COLS as i8 && ball.y >= 0 && ball.y < ROWS as i8 {
            fb[ball.y as usize][ball.x as usize] = true;
        }

        // Draw paddle (bottom row)
        for px in 0..PADDLE_W {
            let col = paddle.x + px as i8;
            if col >= 0 && col < COLS as i8 {
                fb[ROWS - 1][col as usize] = true;
            }
        }

        // Draw bricks
        for r in 0..BRICK_ROWS {
            for c in 0..BRICK_COLS {
                if bricks.grid[r][c] {
                    // Brick pixel column: col * 2 + 1
                    let px = c * 2 + 1;
                    if px < COLS {
                        fb[r][px] = true;
                    }
                    // Second cell of brick
                    if px + 1 < COLS {
                        fb[r][px + 1] = true;
                    }
                }
            }
        }

        // Draw state text
        if state == 1 {
            // Show "!" at center for ball lost
            fb[ROWS - 2][COLS / 2] = true;
        } else if state == 2 {
            // Show "!" at center for win
            fb[ROWS - 2][COLS / 2] = true;
        }

        // Render
        led.scan(&fb, 10).await;
        frame += 1;
    }
}
