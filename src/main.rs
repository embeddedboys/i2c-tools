#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use ch32_hal::gpio::Speed;
use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::gpio::{AnyPin, Level, Output, Pin};
use hal::println;
use {ch32_hal as hal, panic_halt as _};

#[embassy_executor::task]
async fn blink(p:ch32_hal::Peripherals, interval_ms: u64) {
    // let mut led = Output::new(pin, Level::Low, Speed::Low);
    let _ = Output::new(p.PB5, Level::High, Speed::Low);
    let _ = Output::new(p.PB4, Level::High, Speed::Low);
    let _ = Output::new(p.PB3, Level::High, Speed::Low);
    let _ = Output::new(p.PA15, Level::High, Speed::Low);
    let _ = Output::new(p.PA14, Level::High, Speed::Low);
    let _ = Output::new(p.PA13, Level::High, Speed::Low);
    let _ = Output::new(p.PA12, Level::High, Speed::Low);
    let _ = Output::new(p.PA11, Level::High, Speed::Low);
    let _ = Output::new(p.PA10, Level::High, Speed::Low);
    let _ = Output::new(p.PA9, Level::High, Speed::Low);
    let _ = Output::new(p.PA8, Level::High, Speed::Low);
    let _ = Output::new(p.PB15, Level::High, Speed::Low);
    let _ = Output::new(p.PB14, Level::High, Speed::Low);
    let _ = Output::new(p.PB13, Level::High, Speed::Low);
    let _ = Output::new(p.PB12, Level::High, Speed::Low);
    let _ = Output::new(p.PB11, Level::High, Speed::Low);

    let mut leds = [
        Output::new(p.PA3.degrade(), Level::Low, Speed::Low),
        Output::new(p.PA4.degrade(), Level::Low, Speed::Low),
        Output::new(p.PA5.degrade(), Level::Low, Speed::Low),
        Output::new(p.PA6.degrade(), Level::Low, Speed::Low),
        Output::new(p.PA7.degrade(), Level::Low, Speed::Low),
        Output::new(p.PB0.degrade(), Level::Low, Speed::Low),
        Output::new(p.PB1.degrade(), Level::Low, Speed::Low),
        Output::new(p.PB10.degrade(), Level::Low, Speed::Low),
    ];

    loop {
        for led in leds.iter_mut() {
            led.set_high();
        }
        Timer::after_millis(interval_ms).await;
        for led in leds.iter_mut() {
            led.set_low();
        }
        Timer::after_millis(interval_ms).await;
    }
}

#[embassy_executor::main(entry = "qingke_rt::entry")]
async fn main(spawner: Spawner) -> ! {
    // hal::debug::SDIPrint::enable();
    let p: ch32_hal::Peripherals = hal::init(hal::Config::default());
    hal::embassy::init();

    // Adjust the LED GPIO according to your board
    spawner.spawn(blink(p, 200)).unwrap();

    loop {
        Timer::after_millis(1000).await;
        println!("tick");
    }
}
