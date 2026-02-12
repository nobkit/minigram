use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    channel::Channel,
    mutex::Mutex,
};
use esp_hal::{Async, i2c::master::I2c};
use minigram_1_ui::{
    main_menu::draw_main_menu,
    scale::{draw_paused, draw_scale, draw_timer},
};
use ssd1306::{I2CDisplayInterface, Ssd1306Async, mode::BufferedGraphicsModeAsync, prelude::*};
use static_cell::StaticCell;

pub type Display<'a> = Ssd1306Async<
    I2CInterface<
        embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice<
            'a,
            NoopRawMutex,
            esp_hal::i2c::master::I2c<'a, Async>,
        >,
    >,
    ssd1306::prelude::DisplaySize128x64,
    BufferedGraphicsModeAsync<ssd1306::prelude::DisplaySize128x64>,
>;

pub type Displays = (
    &'static Mutex<CriticalSectionRawMutex, Display<'static>>,
    &'static Mutex<CriticalSectionRawMutex, Display<'static>>,
);

static DISPLAY_LEFT: StaticCell<Mutex<CriticalSectionRawMutex, Display<'static>>> =
    StaticCell::new();
static DISPLAY_RIGHT: StaticCell<Mutex<CriticalSectionRawMutex, Display<'static>>> =
    StaticCell::new();

pub async fn initialize_displays(
    i2c_bus: &'static Mutex<NoopRawMutex, I2c<'static, Async>>,
) -> Displays {
    let dev1 = I2cDevice::new(i2c_bus);
    let dev2 = I2cDevice::new(i2c_bus);

    let interface = I2CDisplayInterface::new_custom_address(dev1, 0x3C);

    let interface2 = I2CDisplayInterface::new_custom_address(dev2, 0x3D);

    let mut display_left =
        Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
            .into_buffered_graphics_mode();

    display_left.init().await.unwrap();

    let mut display_right =
        Ssd1306Async::new(interface2, DisplaySize128x64, DisplayRotation::Rotate180)
            .into_buffered_graphics_mode();

    display_right.init().await.unwrap();

    display_left.flush().await.unwrap();
    display_right.flush().await.unwrap();

    let static_display_right = DISPLAY_RIGHT.init(Mutex::new(display_right));
    let static_display_left = DISPLAY_LEFT.init(Mutex::new(display_left));

    return (static_display_left, static_display_right);
}

pub enum DisplayCommand {
    Scale { weight: u64 },
    Timer { time: u64 },
    Paused { paused: bool },
    MainMenu,
    Clear { left: bool, right: bool },
}

pub static DISPLAY_CMD: Channel<CriticalSectionRawMutex, DisplayCommand, 16> = Channel::new();

#[embassy_executor::task]
pub async fn display(displays: Displays) {
    let (left, right) = displays;
    loop {
        let mut l = left.lock().await;
        let mut r = right.lock().await;
        match DISPLAY_CMD.receive().await {
            DisplayCommand::Scale { weight } => {
                draw_scale(&mut *l, weight);
            }
            DisplayCommand::Paused { paused } => {
                draw_paused(&mut *r, paused);
            }
            DisplayCommand::Timer { time } => {
                draw_timer(&mut *r, time);
            }
            DisplayCommand::MainMenu => {
                draw_main_menu(&mut *l);
            }
            DisplayCommand::Clear { left, right } => {
                if left {
                    l.clear_buffer();
                }
                if right {
                    r.clear_buffer();
                }
            }
        }
        l.flush().await.unwrap();
        r.flush().await.unwrap();
    }
}
