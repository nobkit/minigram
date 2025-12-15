#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    mono_font::ascii::*,
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_hal::{
    clock::CpuClock,
    gpio::Pin,
    i2c::{self, master::I2c},
    time::Rate,
    timer::timg::TimerGroup,
};
use minigram_1::input::{
    LEFT_BUTTON_CHANNEL, RIGHT_BUTTON_CHANNEL, handle_button, handle_gestures,
};
use panic_rtt_target as _;
use ssd1306::{I2CDisplayInterface, Ssd1306Async, prelude::*};

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    let i2c = I2c::new(
        peripherals.I2C0,
        i2c::master::Config::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO6)
    .with_scl(peripherals.GPIO7)
    .into_async();

    let bus = Mutex::<NoopRawMutex, _>::new(i2c);
    let dev1 = I2cDevice::new(&bus);
    let dev2 = I2cDevice::new(&bus);

    let interface = I2CDisplayInterface::new_custom_address(dev1, 0x3C);

    let interface2 = I2CDisplayInterface::new_custom_address(dev2, 0x3D);

    let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();

    display.init().await.unwrap();

    let mut display2 = Ssd1306Async::new(interface2, DisplaySize128x64, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();
    display2.init().await.unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Screen One", Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    Text::with_baseline("Screen Two", Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display2)
        .unwrap();

    display.flush().await.unwrap();
    display2.flush().await.unwrap();

    spawner
        .spawn(handle_button(
            peripherals.GPIO8.degrade(),
            RIGHT_BUTTON_CHANNEL.sender(),
        ))
        .unwrap();
    spawner
        .spawn(handle_button(
            peripherals.GPIO20.degrade(),
            LEFT_BUTTON_CHANNEL.sender(),
        ))
        .unwrap();

    spawner.spawn(handle_gestures()).unwrap();
}
