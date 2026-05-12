#![no_std]
#![no_main]

use embassy_executor::Spawner;
use i2c_tools::{COLS, Fb, LedMatrix, Rng, ROWS};
use panic_halt as _;

const MAX_LEN: usize = 64;

#[derive(Copy, Clone, PartialEq)]
struct Pos {
    x: i8,
    y: i8,
}

impl Pos {
    fn step(self, dx: i8, dy: i8) -> Self {
        Self {
            x: (self.x + dx + COLS as i8) % COLS as i8,
            y: (self.y + dy + ROWS as i8) % ROWS as i8,
        }
    }
}

struct Snake {
    body: [Pos; MAX_LEN],
    len: usize,
    dx: i8,
    dy: i8,
}

impl Snake {
    fn new() -> Self {
        let mut body = [Pos { x: 0, y: 0 }; MAX_LEN];
        body[0] = Pos { x: 8, y: 4 };
        body[1] = Pos { x: 7, y: 4 };
        body[2] = Pos { x: 6, y: 4 };
        Self {
            body,
            len: 3,
            dx: 1,
            dy: 0,
        }
    }

    fn head(&self) -> Pos {
        self.body[0]
    }

    fn occupies(&self, p: Pos) -> bool {
        for i in 0..self.len {
            if self.body[i] == p {
                return true;
            }
        }
        false
    }

    /// Check if moving in (dx, dy) would hit the snake body
    fn would_hit(&self, dx: i8, dy: i8) -> bool {
        let next = self.head().step(dx, dy);
        // Don't check tail — it will move away (unless we just ate)
        for i in 0..self.len - 1 {
            if self.body[i] == next {
                return true;
            }
        }
        false
    }

    fn step(&mut self) {
        for i in (1..self.len).rev() {
            self.body[i] = self.body[i - 1];
        }
        self.body[0] = self.head().step(self.dx, self.dy);
    }

    fn grow(&mut self) {
        if self.len < MAX_LEN {
            self.body[self.len] = self.body[self.len - 1];
            self.len += 1;
        }
    }

    fn hits_self(&self) -> bool {
        for i in 1..self.len {
            if self.body[i] == self.body[0] {
                return true;
            }
        }
        false
    }
}

fn spawn_food(rng: &mut Rng, snake: &Snake) -> Pos {
    loop {
        let p = Pos {
            x: (rng.next() as i8).rem_euclid(COLS as i8),
            y: (rng.next() as i8).rem_euclid(ROWS as i8),
        };
        if !snake.occupies(p) {
            return p;
        }
    }
}

/// Choose direction towards food, avoiding self-collision.
/// Returns (dx, dy) or None if boxed in.
fn choose_dir(snake: &Snake, food: Pos) -> Option<(i8, i8)> {
    let head = snake.head();

    // All 4 possible directions
    let dirs: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    // Don't reverse
    let reverse = (-snake.dx, -snake.dy);

    let mut best_dir: Option<(i8, i8)> = None;
    let mut best_dist: i16 = i16::MAX;

    for &(dx, dy) in &dirs {
        // Skip reverse direction
        if (dx, dy) == reverse {
            continue;
        }
        // Skip self-collision
        if snake.would_hit(dx, dy) {
            continue;
        }
        // Manhattan distance to food after this move
        let next = head.step(dx, dy);
        let dist = (next.x - food.x).unsigned_abs() as i16
            + (next.y - food.y).unsigned_abs() as i16;
        if dist < best_dist {
            best_dist = dist;
            best_dir = Some((dx, dy));
        }
    }

    best_dir
}

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(_spawner: Spawner) -> ! {
    let mut config = ch32_hal::Config::default();
    config.rcc = ch32_hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p = ch32_hal::init(config);

    let mut led = LedMatrix::new(p);
    let mut rng = Rng(0x1234_5678);

    let mut snake = Snake::new();
    let mut food = spawn_food(&mut rng, &snake);
    let mut frame: u32 = 0;

    loop {
        // Decide direction each move
        if frame % 15 == 0 {
            if let Some((dx, dy)) = choose_dir(&snake, food) {
                snake.dx = dx;
                snake.dy = dy;
            }

            snake.step();

            if snake.head() == food {
                snake.grow();
                food = spawn_food(&mut rng, &snake);
            }

            if snake.hits_self() {
                snake = Snake::new();
                food = spawn_food(&mut rng, &snake);
            }
        }

        // Build framebuffer
        let mut fb: Fb = [[false; COLS]; ROWS];
        for i in 0..snake.len {
            fb[snake.body[i].y as usize][snake.body[i].x as usize] = true;
        }
        fb[food.y as usize][food.x as usize] = true;

        led.scan_once(&fb).await;
        frame = frame.wrapping_add(1);
    }
}
