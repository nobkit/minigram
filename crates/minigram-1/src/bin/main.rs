#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::{cell::RefCell, pin};

// use bt_hci::controller::ExternalController;
use defmt::info;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    channel::{Channel, Sender},
    mutex::Mutex,
};
use embassy_time::{Duration, Instant, Timer, WithTimeout};
use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    mono_font::ascii::*,
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Baseline, Text},
};
use esp_hal::{
    clock::CpuClock,
    gpio::{AnyPin, Input, InputConfig, Pin, Pull, interconnect::PeripheralInput},
    i2c::{self, master::I2c},
    time::Rate,
    timer::timg::TimerGroup,
};
// use esp_radio::ble::controller::BleConnector;
use panic_rtt_target as _;
use ssd1306::{I2CDisplayInterface, Ssd1306Async, prelude::*};
// use trouble_host::prelude::*;

extern crate alloc;

// const CONNECTIONS_MAX: usize = 1;
// const L2CAP_CHANNELS_MAX: usize = 1;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

enum ButtonId {
    Left = -1,
    Right = 1,
}

#[derive(Clone, Copy, PartialEq, defmt::Format)]
enum ButtonEvent {
    Click,
    DoubleClick,
    Hold,
}

static LEFT_BUTTON_CHANNEL: Channel<CriticalSectionRawMutex, ButtonEvent, 10> = Channel::new();
static RIGHT_BUTTON_CHANNEL: Channel<CriticalSectionRawMutex, ButtonEvent, 10> = Channel::new();

#[embassy_executor::task(pool_size = 2)]
async fn handle_button(
    pin: AnyPin<'static>,
    sender: Sender<'static, CriticalSectionRawMutex, ButtonEvent, 10>,
) {
    const HOLD_TIMEOUT: Duration = Duration::from_secs(1);
    const DOUBLE_CLICK_TIMEOUT: Duration = Duration::from_millis(100);

    let mut button = Input::new(pin, InputConfig::default().with_pull(Pull::Up));

    loop {
        Timer::after_millis(20).await;
        button.wait_for_low().await;

        Timer::after_millis(20).await;
        match button.wait_for_high().with_timeout(HOLD_TIMEOUT).await {
            Err(_) => {
                sender.send(ButtonEvent::Hold).await;

                button.wait_for_high().await;
                continue;
            }
            _ => (),
        }

        Timer::after_millis(20).await;
        match button
            .wait_for_low()
            .with_timeout(DOUBLE_CLICK_TIMEOUT)
            .await
        {
            Ok(_) => {
                Timer::after_millis(20).await;
                button.wait_for_high().await;

                sender.send(ButtonEvent::DoubleClick).await
            }
            Err(_) => sender.send(ButtonEvent::Click).await,
        }
    }
}

#[derive(Clone, Copy, PartialEq, defmt::Format)]
enum InputEvent {
    Left(ButtonEvent),
    Right(ButtonEvent),
    Both(ButtonEvent),
}

#[embassy_executor::task]
async fn handle_gestures() {
    const BOTH_PRESSED_TIMEOUT: Duration = Duration::from_millis(50);

    loop {
        let (button_id, button_event, other_channel) = match select(
            LEFT_BUTTON_CHANNEL.receive(),
            RIGHT_BUTTON_CHANNEL.receive(),
        )
        .await
        {
            Either::First(be) => (ButtonId::Left, be, &RIGHT_BUTTON_CHANNEL),
            Either::Second(be) => (ButtonId::Right, be, &LEFT_BUTTON_CHANNEL),
        };

        match other_channel
            .receive()
            .with_timeout(BOTH_PRESSED_TIMEOUT)
            .await
        {
            Ok(be) if be == button_event => {
                info!("{}", InputEvent::Both(button_event));
            }
            result => match button_id {
                ButtonId::Left => {
                    info!("{}", InputEvent::Left(button_event));
                    if let Some(other_button_event) = result.ok() {
                        info!("{}", InputEvent::Right(other_button_event));
                    }
                }
                ButtonId::Right => {
                    info!("{}", InputEvent::Right(button_event));
                    if let Some(other_button_event) = result.ok() {
                        info!("{}", InputEvent::Left(other_button_event));
                    }
                }
            },
        };
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // generator version: 1.0.1

    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);
    // esp_alloc::heap_allocator!(size: 64 * 1024);

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

    // loop {
    //     Timer::after(Duration::from_secs(1)).await;
    // }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}
