#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use core::fmt::Write;

use ch32_hal::gpio::Speed;
use ch32_hal::i2c::{Config, I2c};
use hal::{pac, println};
use embassy_executor::Spawner;
use embassy_time::{Delay, Duration, Timer};
use {ch32_hal as hal, panic_halt as _};
use hal::gpio::{AnyPin, Level, Output, Pin};

#[embassy_executor::task]
async fn blink(p:ch32_hal::Peripherals, led: u8, interval_ms: u64) {

    let mut columns = [
        Output::new(p.PB5, Level::Low, Speed::Low),
        Output::new(p.PB4, Level::Low, Speed::Low),
        Output::new(p.PB3, Level::Low, Speed::Low),
        Output::new(p.PA15, Level::Low, Speed::Low),
        Output::new(p.PA14, Level::Low, Speed::Low),
        Output::new(p.PA13, Level::Low, Speed::Low),
        Output::new(p.PA12, Level::Low, Speed::Low),
        Output::new(p.PA11, Level::Low, Speed::Low),
        Output::new(p.PA10, Level::Low, Speed::Low),
        Output::new(p.PA9, Level::Low, Speed::Low),
        Output::new(p.PA8, Level::Low, Speed::Low),
        Output::new(p.PB15, Level::Low, Speed::Low),
        Output::new(p.PB14, Level::Low, Speed::Low),
        Output::new(p.PB13, Level::Low, Speed::Low),
        Output::new(p.PB12, Level::Low, Speed::Low),
        Output::new(p.PB11, Level::Low, Speed::Low),
    ];

    let mut rows = [
        Output::new(p.PA3.degrade(), Level::Low, Speed::Low),
        Output::new(p.PA4.degrade(), Level::Low, Speed::Low),
        Output::new(p.PA5.degrade(), Level::Low, Speed::Low),
        Output::new(p.PA6.degrade(), Level::Low, Speed::Low),
        Output::new(p.PA7.degrade(), Level::Low, Speed::Low),
        Output::new(p.PB0.degrade(), Level::Low, Speed::Low),
        Output::new(p.PB1.degrade(), Level::Low, Speed::Low),
        Output::new(p.PB10.degrade(), Level::Low, Speed::Low),
    ];

    let row = (led / 16) as usize;
    let col = (led % 16) as usize;

    rows[row as usize].set_high();

    loop {
        println!("blink");
        columns[col].set_high();
        Timer::after(Duration::from_millis(interval_ms)).await;
        columns[col].set_low();
        Timer::after(Duration::from_millis(interval_ms)).await;
    }
}

#[embassy_executor::main(entry = "qingke_rt::entry")]
async fn main(spawner: Spawner) -> ! {
    hal::debug::SDIPrint::enable();
    let mut config = hal::Config::default();
    config.rcc = hal::rcc::Config::SYSCLK_FREQ_144MHZ_HSE;
    let p: ch32_hal::Peripherals = hal::init(config);
    hal::embassy::init();

    let scl = p.PB8;
    let sda = p.PB9;

    let mut i2c = I2c::new_blocking(p.I2C1, scl, sda, ch32_hal::time::Hertz(400_000), Config::default());

    let mut addr: u8 = 0;

    'outer: for i in (0..128).step_by(16) {
        for j in 0..16 {
            match i2c.blocking_write(i+j, &[0x00]) {
                Ok(_) => {
                    addr = i+j;
                    break 'outer;
                }
                Err(_) => {}
            }
        }
    }

    // spawner.spawn(blink(p, addr, 100)).unwrap();
    // let mut last = pac::SYSTICK.cnt().read();
    // loop {
    //     Timer::after_millis(1000).await;
    //     let cnt = pac::SYSTICK.cnt().read();
    //     let elapsed = cnt.wrapping_sub(last);
    //     println!("tick");
    //     println!("systick: {}", elapsed);
    // }
    let mut columns = [
        Output::new(p.PB5, Level::Low, Speed::Low),
        Output::new(p.PB4, Level::Low, Speed::Low),
        Output::new(p.PB3, Level::Low, Speed::Low),
        Output::new(p.PA15, Level::Low, Speed::Low),
        Output::new(p.PA14, Level::Low, Speed::Low),
        Output::new(p.PA13, Level::Low, Speed::Low),
        Output::new(p.PA12, Level::Low, Speed::Low),
        Output::new(p.PA11, Level::Low, Speed::Low),
        Output::new(p.PA10, Level::Low, Speed::Low),
        Output::new(p.PA9, Level::Low, Speed::Low),
        Output::new(p.PA8, Level::Low, Speed::Low),
        Output::new(p.PB15, Level::Low, Speed::Low),
        Output::new(p.PB14, Level::Low, Speed::Low),
        Output::new(p.PB13, Level::Low, Speed::Low),
        Output::new(p.PB12, Level::Low, Speed::Low),
        Output::new(p.PB11, Level::Low, Speed::Low),
    ];

    let mut rows = [
        Output::new(p.PA3.degrade(), Level::Low, Speed::Low),
        Output::new(p.PA4.degrade(), Level::Low, Speed::Low),
        Output::new(p.PA5.degrade(), Level::Low, Speed::Low),
        Output::new(p.PA6.degrade(), Level::Low, Speed::Low),
        Output::new(p.PA7.degrade(), Level::Low, Speed::Low),
        Output::new(p.PB0.degrade(), Level::Low, Speed::Low),
        Output::new(p.PB1.degrade(), Level::Low, Speed::Low),
        Output::new(p.PB10.degrade(), Level::Low, Speed::Low),
    ];

    rows.reverse();

    let row = (addr / 16) as usize;
    let col = (addr % 16) as usize;

    println!("will blink row {} col {}", row, col);
    rows[row].set_high();
    loop {
        columns[col].set_high();
        Timer::after(Duration::from_millis(500)).await;
        columns[col].set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}