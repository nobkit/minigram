use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle, StyledDrawable},
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
    sdl2::Keycode,
};
use minigram_1_ui::settings_menu::{draw_settings_menu_l, draw_settings_menu_r};

fn draw(display: &mut SimulatorDisplay<BinaryColor>, item: u32) {
    display.clear(BinaryColor::Off).unwrap();

    // Draw a roughly 17-pixel gap between the screens
    Line::new(Point::new(128, 0), Point::new(128, 64))
        .draw_styled(&PrimitiveStyle::with_stroke(BinaryColor::On, 1), display);
    Line::new(Point::new(128 + 16, 0), Point::new(128 + 16, 64))
        .draw_styled(&PrimitiveStyle::with_stroke(BinaryColor::On, 1), display);

    {
        let mut left = display.cropped(&Rectangle::new(Point::new(0, 0), Size::new(128, 64)));
        draw_settings_menu_l(&mut left, item);
    }
    {
        let mut right =
            display.cropped(&Rectangle::new(Point::new(128 + 17, 0), Size::new(128, 64)));
        draw_settings_menu_r(&mut right, item);
    }
}

fn main() -> Result<(), core::convert::Infallible> {
    let mut display = SimulatorDisplay::<BinaryColor>::new(Size::new(256 + 17, 64));

    let mut item: i32 = 0;
    draw(&mut display, item as u32);

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledWhite)
        .scale(3)
        .build();
    let mut window = Window::new("Minigram 1", &output_settings);

    'running: loop {
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => {
                    let delta = match keycode {
                        Keycode::Left => -1,
                        Keycode::Right => 1,
                        _ => 0,
                    };
                    if delta != 0 {
                        item = (item + delta).rem_euclid(3);
                        draw(&mut display, item as u32);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
