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
    icons::{BOX52X52, CALIBRATE52X52, WIFI52X52},
    utils::{LEFT_ORIGIN, RIGHT_ORIGIN, draw_highlight},
};

pub fn draw_settings_menu_l(display: &mut impl DeviceDisplay, item: u32) {
    display.clear(BinaryColor::Off).unwrap();
    if item == 0 {
        draw_highlight(display, LEFT_ORIGIN);
        Text::with_text_style(
            "Wi-Fi",
            Point::new(27, 54),
            TEXT_XS!(BinaryColor::Off),
            TextStyleBuilder::new()
                .alignment(Alignment::Center)
                .baseline(Baseline::Top)
                .build(),
        )
        .draw(display)
        .unwrap();
        CastImage::new(&WIFI52X52, Point::new(2, 2), BinaryColor::Off)
            .draw(display)
            .unwrap();
    } else {
        Text::with_text_style(
            "Wi-Fi",
            Point::new(27, 54),
            TEXT_XS!(),
            TextStyleBuilder::new()
                .alignment(Alignment::Center)
                .baseline(Baseline::Top)
                .build(),
        )
        .draw(display)
        .unwrap();
        Image::new(&WIFI52X52, Point::new(2, 2))
            .draw(display)
            .unwrap();
    }

    if item == 1 {
        draw_highlight(display, RIGHT_ORIGIN);
        Text::with_text_style(
            "Calibration",
            Point::new(99, 54),
            TEXT_XS!(BinaryColor::Off),
            TextStyleBuilder::new()
                .alignment(Alignment::Center)
                .baseline(Baseline::Top)
                .build(),
        )
        .draw(display)
        .unwrap();
        CastImage::new(&CALIBRATE52X52, Point::new(74, 2), BinaryColor::Off)
            .draw(display)
            .unwrap();
    } else {
        Text::with_text_style(
            "Calibration",
            Point::new(99, 54),
            TEXT_XS!(),
            TextStyleBuilder::new()
                .alignment(Alignment::Center)
                .baseline(Baseline::Top)
                .build(),
        )
        .draw(display)
        .unwrap();
        Image::new(&CALIBRATE52X52, Point::new(74, 2))
            .draw(display)
            .unwrap();
    }
}

pub fn draw_settings_menu_r(display: &mut impl DeviceDisplay, item: u32) {
    display.clear(BinaryColor::Off).unwrap();
    if item == 2 {
        draw_highlight(display, LEFT_ORIGIN);
        Text::with_text_style(
            "Update",
            Point::new(27, 54),
            TEXT_XS!(BinaryColor::Off),
            TextStyleBuilder::new()
                .alignment(Alignment::Center)
                .baseline(Baseline::Top)
                .build(),
        )
        .draw(display)
        .unwrap();
        CastImage::new(&BOX52X52, Point::new(2, 2), BinaryColor::Off)
            .draw(display)
            .unwrap();
    } else {
        Text::with_text_style(
            "Update",
            Point::new(27, 54),
            TEXT_XS!(),
            TextStyleBuilder::new()
                .alignment(Alignment::Center)
                .baseline(Baseline::Top)
                .build(),
        )
        .draw(display)
        .unwrap();
        Image::new(&BOX52X52, Point::new(2, 2))
            .draw(display)
            .unwrap();
    }
}
