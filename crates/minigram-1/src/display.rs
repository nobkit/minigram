use defmt::info;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    lazy_lock::LazyLock,
    mutex::Mutex,
    signal::Signal,
};
use embedded_graphics::{
    geometry::AnchorPoint,
    mono_font::{MonoTextStyleBuilder, ascii::*},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};
use esp_hal::{Async, i2c::master::I2c};
use minigram_1_ui::scale::{draw_scale, draw_timer};
use ssd1306::{I2CDisplayInterface, Ssd1306Async, mode::BufferedGraphicsModeAsync, prelude::*};
use static_cell::StaticCell;
use tinybmp::Bmp;
use u8g2_fonts::{
    U8g2TextStyle,
    fonts::{u8g2_font_logisoso16_tf, u8g2_font_logisoso38_tf},
};

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
    Timer { time: u64, paused: bool },
}

pub static DISPLAY_CMD: Signal<CriticalSectionRawMutex, DisplayCommand> = Signal::new();

#[embassy_executor::task]
pub async fn display(displays: Displays) {
    let (mut left, mut right) = displays;
    loop {
        let mut l = left.lock().await;
        let mut r = right.lock().await;
        match DISPLAY_CMD.wait().await {
            DisplayCommand::Scale { weight } => {
                draw_scale(&mut *l, weight);
            }
            DisplayCommand::Timer { time, paused } => {
                draw_timer(&mut *r, time, paused);
            }
        }
        l.flush().await.unwrap();
        r.flush().await.unwrap();
    }
}
