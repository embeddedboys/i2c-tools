# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Embedded Rust firmware for a CH32V203 (RISC-V) MCU that scans the I2C bus and displays results on an 8x16 LED matrix. The project also includes display examples (SSD1306 OLED, various games).

- **Target**: `riscv32imac-unknown-none-elf` (CH32V203 RISC-V MCU)
- **Toolchain**: nightly with `rust-src`
- **HAL**: `ch32-hal` with embassy async runtime
- **License**: MIT OR Apache-2.0

## Key Files

- `src/main.rs` — Main I2C scanner: scans all 128 I2C addresses and blinks the corresponding LED on the 8x16 matrix
- `src/lib.rs` — `LedMatrix` driver (8x16 multiplexed matrix with `scan_once`/`scan` methods), xorshift32 RNG, 5x8 bitmap font (`font` module)
- `examples/` — Standalone examples: `i2c-ssd1306.rs`, `snake.rs`, `tetris.rs`, `dino.rs`, `starry-sky.rs`, `text-scroll.rs`, `blinky.rs`, `bouncing-ball.rs`
- `Cargo.toml` — Package config, `harness = false` (no_std embedded, no built-in test harness)
- `rust-toolchain.toml` — nightly toolchain with `riscv32imc-unknown-none-elf` target

## Commands

```bash
# Install flash tool
cargo install --locked wlink

# Build release
cargo build -r

# Flash via WCH-Link probe
cargo run -r

# Run a specific example
cargo run -r --example i2c-ssd1306
cargo run -r --example snake
```

## Architecture

- `#![no_std]` / `#![no_main]` — baremetal embedded, no OS
- Embassy async executor spins as the runtime; `#[embassy_executor::main]` is the entry point
- I2C uses blocking mode (`I2c::new_blocking`) on I2C1 (PB8=SCL, PB9=SDA, 400kHz fast mode)
- LED matrix: 16 column pins + 8 row pins, multiplexed scanning via `LedMatrix::scan_once`/`scan` in lib.rs
- Examples share the same init pattern: configure 144MHz HSE, init HAL, then run application logic
- All examples are `[[bin]]` targets with `harness = false`
