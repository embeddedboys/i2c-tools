#![no_std]
#![no_main]

use embassy_executor::Spawner;
use i2c_tools::{COLS, Fb, LedMatrix, Rng, ROWS};
use panic_halt as _;

/// Maximum length the snake can grow to
const MAX_LEN: usize = 64;
/// Number of frames between each snake move
const MOVE_INTERVAL: u32 = 15;
/// Total grid size = columns * rows
const GRID_SIZE: usize = COLS * ROWS;
/// Four possible movement directions: right, left, down, up
const DIRECTIONS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Represents a position on the LED matrix grid
#[derive(Copy, Clone, PartialEq)]
struct Pos {
    x: i8,
    y: i8,
}

impl Pos {
    /// Calculate a new position by stepping in the given direction
    /// Handles grid wrapping using modulo arithmetic
    fn step(self, dx: i8, dy: i8) -> Self {
        Self {
            x: (self.x + dx + COLS as i8) % COLS as i8,
            y: (self.y + dy + ROWS as i8) % ROWS as i8,
        }
    }

    /// Calculate Manhattan distance to another position
    fn distance_to(self, other: Pos) -> i16 {
        (self.x - other.x).unsigned_abs() as i16 + (self.y - other.y).unsigned_abs() as i16
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
        // Start with snake facing right in the middle of the grid
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

    /// Get the head position of the snake
    fn head(&self) -> Pos {
        self.body[0]
    }

    /// Get the tail position of the snake
    fn tail(&self) -> Pos {
        self.body[self.len - 1]
    }

    /// Check if a position is occupied by any snake body segment
    fn occupies(&self, p: Pos) -> bool {
        for i in 0..self.len {
            if self.body[i] == p {
                return true;
            }
        }
        false
    }

    /// Check if moving in direction (dx, dy) would result in collision
    /// Excludes tail from check since it will move away when snake steps
    fn would_hit(&self, dx: i8, dy: i8) -> bool {
        let next = self.head().step(dx, dy);
        // Check all body segments except tail (index len-1)
        for i in 0..self.len.saturating_sub(1) {
            if self.body[i] == next {
                return true;
            }
        }
        false
    }

    /// Move the snake one step forward in current direction
    fn step(&mut self) {
        // Shift all body segments backward
        for i in (1..self.len).rev() {
            self.body[i] = self.body[i - 1];
        }
        // Move head to new position
        self.body[0] = self.head().step(self.dx, self.dy);
    }

    /// Grow the snake by one segment (duplicates tail position)
    fn grow(&mut self) {
        if self.len < MAX_LEN {
            self.body[self.len] = self.body[self.len - 1];
            self.len += 1;
        }
    }

    /// Check if snake head has collided with any body segment
    fn hits_self(&self) -> bool {
        for i in 1..self.len {
            if self.body[i] == self.head() {
                return true;
            }
        }
        false
    }
}

/// Spawn food at a random position not occupied by the snake
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

/// Count reachable empty cells from start position using flood fill
/// Excludes snake tail from occupied cells since it will move away
fn flood_count(start: Pos, snake: &Snake) -> usize {
    // Track visited cells
    let mut visited = [[false; COLS]; ROWS];
    
    // Mark snake body as occupied (except tail, which will move)
    for i in 0..snake.len.saturating_sub(1) {
        visited[snake.body[i].y as usize][snake.body[i].x as usize] = true;
    }
    
    let mut count = 0usize;
    // Stack for DFS traversal, size = grid size
    let mut stack = [Pos { x: 0, y: 0 }; GRID_SIZE];
    let mut stack_ptr = 0;
    
    // Initialize with start position
    stack[stack_ptr] = start;
    stack_ptr += 1;
    visited[start.y as usize][start.x as usize] = true;

    // Perform DFS to count all reachable cells
    while stack_ptr > 0 {
        stack_ptr -= 1;
        let p = stack[stack_ptr];
        count += 1;
        
        // Explore all four directions
        for &(dx, dy) in &DIRECTIONS {
            let neighbor = p.step(dx, dy);
            if !visited[neighbor.y as usize][neighbor.x as usize] {
                visited[neighbor.y as usize][neighbor.x as usize] = true;
                stack[stack_ptr] = neighbor;
                stack_ptr += 1;
            }
        }
    }
    
    count
}

/// Represents a possible move with its calculated metrics
#[derive(Copy, Clone)]
struct Move {
    dx: i8,
    dy: i8,
    next: Pos,
    space: usize,
}

/// AI function to choose the best direction for the snake
/// Strategy:
/// 1. Chase food if there's enough space to avoid getting trapped
/// 2. If no safe path to food, chase tail to maximize survival chance
fn choose_dir(snake: &Snake, food: Pos) -> Option<(i8, i8)> {
    let head = snake.head();
    let tail = snake.tail();
    let reverse = (-snake.dx, -snake.dy);
    
    // Collect all valid safe moves
    let mut moves = [Move { dx: 0, dy: 0, next: Pos { x: 0, y: 0 }, space: 0 }; 3];
    let mut num_moves = 0;

    for &(dx, dy) in &DIRECTIONS {
        // Skip reverse direction (can't go backward)
        if (dx, dy) == reverse {
            continue;
        }
        // Skip moves that would cause collision
        if snake.would_hit(dx, dy) {
            continue;
        }
        // Calculate next position and available space
        let next = head.step(dx, dy);
        let space = flood_count(next, snake);
        moves[num_moves] = Move { dx, dy, next, space };
        num_moves += 1;
    }

    // No safe moves available
    if num_moves == 0 {
        return None;
    }

    // Priority 1: Move toward food AND have enough space to survive
    let mut best_move: Option<(i8, i8)> = None;
    let mut best_dist = i16::MAX;
    
    for i in 0..num_moves {
        let m = &moves[i];
        // Only consider moves with enough space to fit the entire snake
        if m.space >= snake.len {
            let dist = m.next.distance_to(food);
            if dist < best_dist {
                best_dist = dist;
                best_move = Some((m.dx, m.dy));
            }
        }
    }
    
    if best_move.is_some() {
        return best_move;
    }

    // Priority 2: Move toward tail (maximize space) to survive
    best_dist = i16::MAX;
    for i in 0..num_moves {
        let m = &moves[i];
        let dist = m.next.distance_to(tail);
        if dist < best_dist {
            best_dist = dist;
            best_move = Some((m.dx, m.dy));
        }
    }
    
    best_move
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
    // Initialize RNG with fixed seed for consistent behavior
    let mut rng = Rng(0x1234_5678);

    // Initialize game state
    let mut snake = Snake::new();
    let mut food = spawn_food(&mut rng, &snake);
    let mut frame: u32 = 0;

    // Main game loop
    loop {
        // Update game logic at fixed intervals
        if frame % MOVE_INTERVAL == 0 {
            // Let AI choose next move direction
            let dir = choose_dir(&snake, food);

            if let Some((dx, dy)) = dir {
                // Apply chosen direction
                snake.dx = dx;
                snake.dy = dy;
            } else {
                // No safe moves available, reset game
                snake = Snake::new();
                food = spawn_food(&mut rng, &snake);
                frame = frame.wrapping_add(1);
                continue;
            }

            // Move snake forward
            snake.step();

            // Check if snake ate food
            if snake.head() == food {
                snake.grow();
                food = spawn_food(&mut rng, &snake);
            }

            // Check for self collision after move/grow
            if snake.hits_self() {
                snake = Snake::new();
                food = spawn_food(&mut rng, &snake);
            }
        }

        // Render frame - draw snake and food on LED matrix
        let mut fb: Fb = [[false; COLS]; ROWS];
        for i in 0..snake.len {
            fb[snake.body[i].y as usize][snake.body[i].x as usize] = true;
        }
        fb[food.y as usize][food.x as usize] = true;

        // Scan LED matrix to display the frame
        led.scan_once(&fb).await;
        frame = frame.wrapping_add(1);
    }
}
