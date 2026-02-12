use embedded_graphics::{
    prelude::*,
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};

use crate::DeviceDisplay;
use crate::text::TEXT_S;

pub fn draw_main_menu(display: &mut impl DeviceDisplay) {
    Text::with_text_style(
        "main menu",
        Point::new(125, 42),
        &TEXT_S,
        TextStyleBuilder::new()
            .alignment(Alignment::Right)
            .baseline(Baseline::Top)
            .build(),
    )
    .draw(display)
    .unwrap();
}
