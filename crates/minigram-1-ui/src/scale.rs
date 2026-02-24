use embedded_graphics::{
    image::Image,
    pixelcolor::*,
    prelude::*,
    primitives::*,
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};
use heapless::{String, format};

use crate::text::{TEXT_L, TEXT_S};
use crate::{DeviceDisplay, icons::PAUSE};

pub fn draw_timer(display: &mut impl DeviceDisplay, seconds: u64) {
    Rectangle::new(Point::new(0, 0), Size::new(103, 44))
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
        .draw(display)
        .unwrap();

    Text::with_text_style(
        (format!("{:02}", seconds / 60).unwrap() as String<2>).as_str(),
        Point::new(0, 2),
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
        Point::new(44, 2),
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
        Point::new(54, 2),
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

pub fn draw_paused(display: &mut impl DeviceDisplay, paused: bool) {
    if paused {
        Image::new(&PAUSE, Point::new(103, 19))
            .draw(display)
            .unwrap();
    } else {
        Rectangle::new(Point::new(103, 19), Size::new(22, 22))
            .draw_styled(&PrimitiveStyle::with_fill(BinaryColor::Off), display)
            .unwrap();
    }
}

pub fn draw_scale(display: &mut impl DeviceDisplay, weight: u64) {
    Rectangle::new(Point::new(0, 0), Size::new(128, 46))
        .draw_styled(&PrimitiveStyle::with_fill(BinaryColor::Off), display)
        .unwrap();
    Text::with_text_style(
        (format!("{}", weight / 10).unwrap() as String<4>).as_str(),
        Point::new(96, 2),
        &TEXT_L,
        TextStyleBuilder::new()
            .alignment(Alignment::Right)
            .baseline(Baseline::Top)
            .build(),
    )
    .draw(display)
    .unwrap();

    Text::with_text_style(
        ".",
        Point::new(93, 2),
        &TEXT_L,
        TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .baseline(Baseline::Top)
            .build(),
    )
    .draw(display)
    .unwrap();

    Text::with_text_style(
        (format!("{}", weight % 10).unwrap() as String<1>).as_str(),
        Point::new(103, 2),
        &TEXT_L,
        TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .baseline(Baseline::Top)
            .build(),
    )
    .draw(display)
    .unwrap();

    Text::with_text_style(
        "grams",
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
