use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    mutex::Mutex,
};
use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    mono_font::ascii::*,
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_hal::{Async, i2c::master::I2c};
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

static DISPLAY_LEFT: StaticCell<Mutex<CriticalSectionRawMutex, Display<'static>>> =
    StaticCell::new();
static DISPLAY_RIGHT: StaticCell<Mutex<CriticalSectionRawMutex, Display<'static>>> =
    StaticCell::new();

pub async fn initialize_displays(
    i2c_bus: &'static Mutex<NoopRawMutex, I2c<'static, Async>>,
) -> (
    &'static Mutex<CriticalSectionRawMutex, Display<'static>>,
    &'static Mutex<CriticalSectionRawMutex, Display<'static>>,
) {
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

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Screen One", Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display_left)
        .unwrap();

    Text::with_baseline("Screen Two", Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display_right)
        .unwrap();

    display_left.flush().await.unwrap();
    display_right.flush().await.unwrap();

    let static_display_right = DISPLAY_RIGHT.init(Mutex::new(display_right));
    let static_display_left = DISPLAY_LEFT.init(Mutex::new(display_left));

    return (static_display_right, static_display_left);
}
