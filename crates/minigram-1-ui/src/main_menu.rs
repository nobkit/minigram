use embedded_graphics::{
    image::Image,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};
use embedded_graphics_colorcast::Image as CastImage;

use crate::{
    DeviceDisplay, TEXT_XS,
    icons::{BOX52X52, COG52X52, POUROVER52X52, SCALE52X52},
};

const HIGHLIGHT_SIZE: Size = Size::new(56, 64);
const LEFT_ORIGIN: Point = Point::new(0, 0);
const RIGHT_ORIGIN: Point = Point::new(72, 0);

fn draw_highlight(display: &mut impl DeviceDisplay, origin: Point) {
    Rectangle::new(origin, HIGHLIGHT_SIZE)
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(display)
        .unwrap();
}

pub fn draw_main_menu_l(display: &mut impl DeviceDisplay, item: u32) {
    display.clear(BinaryColor::Off).unwrap();
    if item == 0 {
        draw_highlight(display, LEFT_ORIGIN);
        Text::with_text_style(
            "Scale",
            Point::new(19, 54),
            TEXT_XS!(BinaryColor::Off),
            TextStyleBuilder::new()
                .alignment(Alignment::Left)
                .baseline(Baseline::Top)
                .build(),
        )
        .draw(display)
        .unwrap();
        CastImage::new(&SCALE52X52, Point::new(2, 2), BinaryColor::Off)
            .draw(display)
            .unwrap();
    } else {
        Text::with_text_style(
            "Scale",
            Point::new(19, 54),
            TEXT_XS!(),
            TextStyleBuilder::new()
                .alignment(Alignment::Left)
                .baseline(Baseline::Top)
                .build(),
        )
        .draw(display)
        .unwrap();
        Image::new(&SCALE52X52, Point::new(2, 2))
            .draw(display)
            .unwrap();
    }

    if item == 1 {
        draw_highlight(display, RIGHT_ORIGIN);
        Text::with_text_style(
            "Pour-Over",
            Point::new(81, 54),
            TEXT_XS!(BinaryColor::Off),
            TextStyleBuilder::new()
                .alignment(Alignment::Left)
                .baseline(Baseline::Top)
                .build(),
        )
        .draw(display)
        .unwrap();
        CastImage::new(&POUROVER52X52, Point::new(74, 2), BinaryColor::Off)
            .draw(display)
            .unwrap();
    } else {
        Text::with_text_style(
            "Pour-Over",
            Point::new(81, 54),
            TEXT_XS!(),
            TextStyleBuilder::new()
                .alignment(Alignment::Left)
                .baseline(Baseline::Top)
                .build(),
        )
        .draw(display)
        .unwrap();
        Image::new(&POUROVER52X52, Point::new(74, 2))
            .draw(display)
            .unwrap();
    }
}

pub fn draw_main_menu_r(display: &mut impl DeviceDisplay, item: u32) {
    display.clear(BinaryColor::Off).unwrap();
    if item == 2 {
        draw_highlight(display, LEFT_ORIGIN);
        Text::with_text_style(
            "Settings",
            Point::new(13, 54),
            TEXT_XS!(BinaryColor::Off),
            TextStyleBuilder::new()
                .alignment(Alignment::Left)
                .baseline(Baseline::Top)
                .build(),
        )
        .draw(display)
        .unwrap();
        CastImage::new(&COG52X52, Point::new(2, 2), BinaryColor::Off)
            .draw(display)
            .unwrap();
    } else {
        Text::with_text_style(
            "Settings",
            Point::new(13, 54),
            TEXT_XS!(),
            TextStyleBuilder::new()
                .alignment(Alignment::Left)
                .baseline(Baseline::Top)
                .build(),
        )
        .draw(display)
        .unwrap();
        Image::new(&COG52X52, Point::new(2, 2))
            .draw(display)
            .unwrap();
    }

    if item == 3 {
        draw_highlight(display, RIGHT_ORIGIN);
        Text::with_text_style(
            "Updates",
            Point::new(86, 54),
            TEXT_XS!(BinaryColor::Off),
            TextStyleBuilder::new()
                .alignment(Alignment::Left)
                .baseline(Baseline::Top)
                .build(),
        )
        .draw(display)
        .unwrap();
        CastImage::new(&BOX52X52, Point::new(74, 2), BinaryColor::Off)
            .draw(display)
            .unwrap();
    } else {
        Text::with_text_style(
            "Updates",
            Point::new(86, 54),
            TEXT_XS!(),
            TextStyleBuilder::new()
                .alignment(Alignment::Left)
                .baseline(Baseline::Top)
                .build(),
        )
        .draw(display)
        .unwrap();
        Image::new(&BOX52X52, Point::new(74, 2))
            .draw(display)
            .unwrap();
    }
}
