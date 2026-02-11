use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X9},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle, StyledDrawable},
    text::Text,
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use minigram_1_ui::scale::draw_timer;

fn main() -> Result<(), core::convert::Infallible> {
    let mut display = SimulatorDisplay::<BinaryColor>::new(Size::new(256 + 17, 64));

    // Draw a roughly 17-pixel gap between the screens

    Line::new(Point::new(128, 0), Point::new(128, 64)).draw_styled(
        &PrimitiveStyle::with_stroke(BinaryColor::On, 1),
        &mut display,
    );
    Line::new(Point::new(128 + 16, 0), Point::new(128 + 16, 64)).draw_styled(
        &PrimitiveStyle::with_stroke(BinaryColor::On, 1),
        &mut display,
    );

    let mut left_display = display.cropped(&Rectangle::new(Point::new(0, 0), Size::new(128, 64)));

    let mut right_display =
        display.cropped(&Rectangle::new(Point::new(128 + 17, 0), Size::new(128, 64)));

    draw_timer(&mut right_display, 124u64, true);

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledWhite)
        .build();
    Window::new("Minigram 1", &output_settings).show_static(&display);

    Ok(())
}
