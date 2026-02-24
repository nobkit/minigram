use embedded_graphics::{
    image::Image,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, StyledDrawable},
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};

use embedded_graphics_colorcast::Image as CastImage;

use crate::{
    DeviceDisplay, TEXT_XS,
    icons::{
        ARROW_LEFT13X16, ARROW_RIGHT13X16, CHECK24X19, DISABLED_OVERLAY128X64, LR13X9, LR31X19,
    },
    text::TEXT_S,
};

#[derive(Clone, Copy, PartialEq)]
pub enum CalibrationState {
    Browse,
    Waiting,
    Complete,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SettingsOption {
    Calibration(CalibrationState),
    Bluetooth,
    WiFi,
    DeviceInfo,
}

fn draw_text_sm(input: &'static str, x: i32, y: i32, display: &mut impl DeviceDisplay) {
    Text::with_text_style(
        input,
        Point::new(x, y),
        &TEXT_S,
        TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .baseline(Baseline::Top)
            .build(),
    )
    .draw(display)
    .unwrap();
}

fn draw_instructions(display: &mut impl DeviceDisplay, disabled: bool) {
    draw_text_sm("place", 7, 2, display);
    draw_text_sm("100g", 57, 2, display);
    draw_text_sm("on", 102, 2, display);
    draw_text_sm("the", 3, 22, display);
    draw_text_sm("scale,", 34, 22, display);
    draw_text_sm("then", 89, 22, display);
    draw_text_sm("hold", 28, 42, display);
    Image::new(&LR31X19, Point::new(68, 42))
        .draw(display)
        .unwrap();
    if disabled {
        CastImage::new(&DISABLED_OVERLAY128X64, Point::new(0, 0), BinaryColor::Off)
            .draw(display)
            .unwrap();
    }
}

fn draw_complete(display: &mut impl DeviceDisplay) {
    Rectangle::new(Point::new(0, 0), Size::new(128, 64))
        .draw_styled(&PrimitiveStyle::with_fill(BinaryColor::Off), display)
        .unwrap();
    Image::new(&CHECK24X19, Point::new(8, 22))
        .draw(display)
        .unwrap();
    draw_text_sm("calibrated", 32, 22, display);
}

pub fn draw_settings_l(display: &mut impl DeviceDisplay, opt: &SettingsOption) {
    Image::new(&ARROW_LEFT13X16, Point::new(2, 24))
        .draw(display)
        .unwrap();

    Image::new(&ARROW_RIGHT13X16, Point::new(113, 24))
        .draw(display)
        .unwrap();

    Rectangle::new(Point::new(0, 41), Size::new(128, 23))
        .draw_styled(&PrimitiveStyle::with_fill(BinaryColor::Off), display)
        .unwrap();

    match opt {
        SettingsOption::Calibration(calibration_state) => {
            draw_text_sm("calibration", 16, 21, display);
            match calibration_state {
                CalibrationState::Browse => {
                    Text::with_text_style(
                        "Press",
                        Point::new(25, 47),
                        TEXT_XS!(),
                        TextStyleBuilder::new()
                            .alignment(Alignment::Left)
                            .baseline(Baseline::Top)
                            .build(),
                    )
                    .draw(display)
                    .unwrap();

                    Image::new(&LR13X9, Point::new(47, 47))
                        .draw(display)
                        .unwrap();

                    Text::with_text_style(
                        "to configure",
                        Point::new(63, 47),
                        TEXT_XS!(),
                        TextStyleBuilder::new()
                            .alignment(Alignment::Left)
                            .baseline(Baseline::Top)
                            .build(),
                    )
                    .draw(display)
                    .unwrap();
                }
                CalibrationState::Waiting => {
                    Text::with_text_style(
                        "Double-press",
                        Point::new(16, 47),
                        TEXT_XS!(),
                        TextStyleBuilder::new()
                            .alignment(Alignment::Left)
                            .baseline(Baseline::Top)
                            .build(),
                    )
                    .draw(display)
                    .unwrap();

                    Image::new(&LR13X9, Point::new(65, 47))
                        .draw(display)
                        .unwrap();

                    Text::with_text_style(
                        "to cancel",
                        Point::new(81, 47),
                        TEXT_XS!(),
                        TextStyleBuilder::new()
                            .alignment(Alignment::Left)
                            .baseline(Baseline::Top)
                            .build(),
                    )
                    .draw(display)
                    .unwrap();
                }
                CalibrationState::Complete => {}
            }
        }
        SettingsOption::Bluetooth => todo!(),
        SettingsOption::WiFi => todo!(),
        SettingsOption::DeviceInfo => todo!(),
    }
}

pub fn draw_settings_r(display: &mut impl DeviceDisplay, opt: &SettingsOption) {
    match opt {
        SettingsOption::Calibration(calibration_state) => match calibration_state {
            CalibrationState::Browse => {
                draw_instructions(display, true);
            }
            CalibrationState::Waiting => {
                draw_instructions(display, false);
            }
            CalibrationState::Complete => {
                draw_complete(display);
            }
        },
        SettingsOption::Bluetooth => todo!(),
        SettingsOption::WiFi => todo!(),
        SettingsOption::DeviceInfo => todo!(),
    }
}
