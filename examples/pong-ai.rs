#![no_std]
#![no_main]

use embassy_executor::Spawner;
use i2c_tools::{COLS, Fb, LedMatrix, Rng, ROWS};
use panic_halt as _;

/// Ball speed (cells per update)
const BALL_SPEED: i8 = 1;
/// AI paddle speed limit (cells per update)
const AI_SPEED: i8 = 1;
/// Frames between ball updates (slower = easier to watch)
const MOVE_INTERVAL: u32 = 10;
/// Frames to wait after scoring before resetting
const RESET_FRAMES: u32 = 60;

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
            dx: BALL_SPEED,
            dy: 1,
        }
    }

    /// Reset ball to center with random direction
    fn reset(&mut self, rng: &mut Rng) {
        self.x = COLS as i8 / 2;
        self.y = ROWS as i8 / 2;
        self.dx = if (rng.next() % 2) == 0 { BALL_SPEED } else { -BALL_SPEED };
        self.dy = ((rng.next() as i8) % 5) - 2;
    }

    /// Move ball one step, clamping to grid bounds
    fn step(&mut self) {
        self.x += self.dx;
        self.y += self.dy;

        // Clamp and bounce off top/bottom
        if self.y < 0 {
            self.y = 0;
            self.dy = -self.dy;
        } else if self.y >= ROWS as i8 {
            self.y = ROWS as i8 - 1;
            self.dy = -self.dy;
        }
    }
}

/// Paddle: 3-cell tall vertical bar
struct Paddle {
    y: i8,
}

impl Paddle {
    fn new() -> Self {
        Self { y: ROWS as i8 / 2 - 1 }
    }

    /// Move paddle toward target Y, limited by AI_SPEED
    fn move_toward(&mut self, target: i8) {
        let diff = target - self.y;
        let abs_diff = diff.abs() as i8;
        if abs_diff > AI_SPEED {
            self.y += diff.signum() * AI_SPEED;
        } else {
            self.y = target;
        }
        if self.y < 0 { self.y = 0; }
        if self.y > ROWS as i8 - 3 { self.y = ROWS as i8 - 3; }
    }

    /// Check if paddle covers position (py)
    fn covers(&self, py: i8) -> bool {
        py >= self.y && py < self.y + 3
    }
}

/// AI strategy: try to make the opponent miss.
/// When ball is moving toward us, track it carefully.
/// When ball is moving away, predict where it will come back or drift to center.
struct AI {
    target: i8,
}

impl AI {
    fn new() -> Self {
        Self {
            target: ROWS as i8 / 2 - 1,
        }
    }

    /// Update AI target based on ball state.
    /// `paddle_side`: 0 = left, 1 = right
    /// `ball_dx`: horizontal direction
    fn update(&mut self, ball: &Ball, paddle_side: u8) {
        if paddle_side == 0 {
            // Left paddle (human side) — wants to hit ball toward right
            // Always try to intercept the ball
            self.target = ball.y - 1;
        } else {
            // Right paddle (AI side) — wants to hit ball toward left
            // When ball coming toward us, track precisely
            // When ball moving away, predict where it bounces off wall and comes back
            if ball.dx > 0 {
                self.target = ball.y - 1;
            } else {
                // Ball going away — predict where it returns
                // Simple: just go to center, it'll be ready when ball comes back
                self.target = ROWS as i8 / 2 - 1;
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
    let mut rng = Rng(0xBEEF_CAFE);

    let mut ball = Ball::new();
    let mut ai_paddle = Paddle::new();
    let mut human_paddle = Paddle::new();
    let mut ai = AI::new();

    // State: 0 = playing, 1 = reset countdown
    let mut state: u8 = 0;
    let mut reset_timer: u32 = 0;

    let mut frame: u32 = 0;

    loop {
        if state == 0 {
            // Update paddles every frame
            ai.update(&ball, 1); // right paddle
            ai_paddle.move_toward(ai.target);
            human_paddle.move_toward(ball.y - 1);

            // Update ball at fixed interval
            if frame % MOVE_INTERVAL == 0 {
                ball.step();

                // Left paddle collision
                if ball.x == 1 && ball.dx < 0 && human_paddle.covers(ball.y) {
                    // Hit the ball toward the right side (opponent side)
                    ball.x = 2;
                    ball.dx = BALL_SPEED; // Always send to opponent
                    // Angle based on hit position
                    ball.dy = ((ball.y as i8) - (human_paddle.y + 1)) * 2;
                    if ball.dy > 3 { ball.dy = 3; }
                    if ball.dy < -3 { ball.dy = -3; }
                } else if ball.x == 0 && ball.dx < 0 {
                    // Ball escaped left
                    state = 1;
                    reset_timer = RESET_FRAMES;
                    ball.reset(&mut rng);
                }

                // Right paddle collision
                if ball.x == (COLS as i8 - 2) && ball.dx > 0 && ai_paddle.covers(ball.y) {
                    // Hit the ball toward the left side (opponent side)
                    ball.x = COLS as i8 - 3;
                    ball.dx = -BALL_SPEED; // Always send to opponent
                    ball.dy = ((ball.y as i8) - (ai_paddle.y + 1)) * 2;
                    if ball.dy > 3 { ball.dy = 3; }
                    if ball.dy < -3 { ball.dy = -3; }
                } else if ball.x == (COLS as i8 - 1) && ball.dx > 0 {
                    // Ball escaped right
                    state = 1;
                    reset_timer = RESET_FRAMES;
                    ball.reset(&mut rng);
                }
            }
        } else {
            // Reset countdown — paddles drift to center
            ai_paddle.move_toward(ROWS as i8 / 2 - 1);
            human_paddle.move_toward(ROWS as i8 / 2 - 1);

            reset_timer -= 1;
            if reset_timer == 0 {
                state = 0;
            }
        }

        // Build framebuffer
        let mut fb: Fb = [[false; COLS]; ROWS];

        // Draw ball
        if ball.x >= 0 && ball.x < COLS as i8 && ball.y >= 0 && ball.y < ROWS as i8 {
            fb[ball.y as usize][ball.x as usize] = true;
        }

        // Draw AI paddle (right side)
        for ry in 0..3 {
            let py = ai_paddle.y + ry;
            if py >= 0 && py < ROWS as i8 {
                fb[py as usize][COLS - 2] = true;
            }
        }

        // Draw Human paddle (left side)
        for ry in 0..3 {
            let py = human_paddle.y + ry;
            if py >= 0 && py < ROWS as i8 {
                fb[py as usize][1] = true;
            }
        }

        // Draw center line
        for r in 0..ROWS {
            fb[r][COLS / 2] = true;
        }

        // Render
        led.scan(&fb, 10).await;
        frame += 1;
    }
}
