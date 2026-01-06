#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};

use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, Output, Pin},
    i2c::{self, master::I2c},
    time::Rate,
    timer::timg::TimerGroup,
};
use loadcell::hx711::HX711;
use minigram_1::{
    display::initialize_displays,
    input::{LEFT_BUTTON_CHANNEL, RIGHT_BUTTON_CHANNEL, handle_button, handle_gestures},
    load_cell::load_cell,
    routes::{ROUTE, RouteName, main_menu::main_menu_route, scale::scale_route},
};
use panic_rtt_target as _;
use static_cell::StaticCell;

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

static I2C_BUS: StaticCell<Mutex<NoopRawMutex, I2c<esp_hal::Async>>> = StaticCell::new();

static LOAD_CELL: StaticCell<Mutex<NoopRawMutex, HX711<Output<'static>, Input<'static>, Delay>>> =
    StaticCell::new();

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
    let i2c: I2c<'_, esp_hal::Async> = I2c::new(
        peripherals.I2C0,
        i2c::master::Config::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO6)
    .with_scl(peripherals.GPIO7)
    .into_async();

    let i2c_bus = I2C_BUS.init(Mutex::new(i2c));

    let (display_left, display_right) = initialize_displays(i2c_bus).await;

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

    ROUTE.sender().send(RouteName::MainMenu);

    spawner.spawn(main_menu_route()).unwrap();
    spawner.spawn(scale_route()).unwrap();

    spawner
        .spawn(load_cell(
            peripherals.GPIO10.degrade(),
            peripherals.GPIO9.degrade(),
        ))
        .unwrap();
}
