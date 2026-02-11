use embedded_graphics::{
    image::Image,
    pixelcolor::*,
    prelude::*,
    primitives::*,
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};
use heapless::{String, format};

use crate::icons::PAUSE;
use crate::text::{TEXT_L, TEXT_S};

pub fn draw_timer<D>(display: &mut D, seconds: u64, paused: bool)
where
    D: DrawTarget<Color = BinaryColor>,
    D::Error: core::fmt::Debug,
{
    Rectangle::new(Point::new(0, 0), Size::new(128, 44))
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
        .draw(display)
        .unwrap();

    if paused {
        Image::new(&PAUSE, Point::new(105, 19))
            .draw(display)
            .unwrap();
    }

    Text::with_text_style(
        (format!("{:02}", seconds / 60).unwrap() as String<2>).as_str(),
        Point::new(1, 3),
        &TEXT_L,
        TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .baseline(Baseline::Top)
            .build(),
    )
    .draw(display)
    .unwrap();

    Text::with_text_style(
        ":",
        Point::new(46, 3),
        &TEXT_L,
        TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .baseline(Baseline::Top)
            .build(),
    )
    .draw(display)
    .unwrap();

    Text::with_text_style(
        (format!("{:02}", seconds % 60).unwrap() as String<2>).as_str(),
        Point::new(56, 3),
        &TEXT_L,
        TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .baseline(Baseline::Top)
            .build(),
    )
    .draw(display)
    .unwrap();

    Text::with_text_style(
        "time",
        Point::new(3, 42),
        &TEXT_S,
        TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .baseline(Baseline::Top)
            .build(),
    )
    .draw(display)
    .unwrap();
}
