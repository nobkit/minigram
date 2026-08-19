use embedded_graphics::{
    image::Image,
    prelude::*,
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};

use heapless::{String, format};

use crate::{
    DeviceDisplay, TEXT_XS,
    icons::{GITHUB_QR58X58, LR13X9, LR31X19},
    text::TEXT_S,
};

#[derive(Clone, Copy, PartialEq)]
pub enum CalibrationState {
    Waiting,
    Complete,
}

#[derive(Clone, PartialEq)]
pub enum WiFiState {
    Connected(String<32>),
    NotConnected,
    /// A phone is pairing over BLE and has to be told this pass key.
    Pairing(u32),
}

/// The settings screens that currently exist. `Update` has no screens yet, so
/// it is deliberately absent here.
#[derive(Clone, PartialEq)]
pub enum SettingsOption {
    WiFi(WiFiState),
    Calibration(CalibrationState),
}

fn draw_double_press_lr_to_go_back(display: &mut impl DeviceDisplay) {
    Text::with_text_style(
        "Double-press",
        Point::new(14, 47),
        TEXT_XS!(),
        TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .baseline(Baseline::Top)
            .build(),
    )
    .draw(display)
    .unwrap();

    Image::new(&LR13X9, Point::new(63, 47))
        .draw(display)
        .unwrap();

    Text::with_text_style(
        "to go back",
        Point::new(79, 47),
        TEXT_XS!(),
        TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .baseline(Baseline::Top)
            .build(),
    )
    .draw(display)
    .unwrap();
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

fn draw_calibrations_instructions(display: &mut impl DeviceDisplay) {
    draw_text_sm("Place", 19, 12, display);
    draw_text_sm("100g", 71, 12, display);
    draw_text_sm("Hold", 26, 35, display);
    Image::new(&LR31X19, Point::new(70, 33))
        .draw(display)
        .unwrap();
}

fn draw_done(display: &mut impl DeviceDisplay) {
    draw_text_sm("Done!", 41, 22, display);
}

fn draw_wifi_l(display: &mut impl DeviceDisplay) {
    draw_text_sm("Wi-Fi", 16, 22, display);
    draw_text_sm("Setup", 66, 22, display);
    draw_double_press_lr_to_go_back(display);
}

/// The BLE pass key the phone has to be told, filling the whole screen so it
/// stays readable from arm's length.
fn draw_pairing_pass_key(display: &mut impl DeviceDisplay, pass_key: u32) {
    Text::with_text_style(
        "Pair code",
        Point::new(64, 8),
        TEXT_XS!(),
        TextStyleBuilder::new()
            .alignment(Alignment::Center)
            .baseline(Baseline::Top)
            .build(),
    )
    .draw(display)
    .unwrap();

    // Bluetooth pass keys are always rendered as six digits, zero padded.
    let digits: String<6> = format!("{:06}", pass_key % 1_000_000).unwrap();

    Text::with_text_style(
        &digits,
        Point::new(64, 24),
        &TEXT_S,
        TextStyleBuilder::new()
            .alignment(Alignment::Center)
            .baseline(Baseline::Top)
            .build(),
    )
    .draw(display)
    .unwrap();
}

fn draw_wifi_r(display: &mut impl DeviceDisplay, state: &WiFiState) {
    if let WiFiState::Pairing(pass_key) = state {
        draw_pairing_pass_key(display, *pass_key);
        return;
    }

    Image::new(&GITHUB_QR58X58, Point::new(67, 3))
        .draw(display)
        .unwrap();

    if let WiFiState::Connected(ssid) = state {
        Text::with_text_style(
            "Connected",
            Point::new(33, 23),
            TEXT_XS!(),
            TextStyleBuilder::new()
                .alignment(Alignment::Center)
                .baseline(Baseline::Top)
                .build(),
        )
        .draw(display)
        .unwrap();

        let mut truncated_ssid = ssid.clone();

        let mut ssid_draw = Text::with_text_style(
            &truncated_ssid,
            Point::new(33, 34),
            TEXT_XS!(),
            TextStyleBuilder::new()
                .alignment(Alignment::Center)
                .baseline(Baseline::Top)
                .build(),
        );

        let mut display_ssid: String<32>;

        for _ in 0..32 {
            truncated_ssid.pop();
            display_ssid = format!("{}...", &truncated_ssid).unwrap();
            ssid_draw = Text::with_text_style(
                display_ssid.as_str(),
                Point::new(33, 34),
                TEXT_XS!(),
                TextStyleBuilder::new()
                    .alignment(Alignment::Center)
                    .baseline(Baseline::Top)
                    .build(),
            );

            if ssid_draw.bounding_box().size.width <= 63 {
                break;
            }
        }

        ssid_draw.draw(display).unwrap();
    } else {
        Text::with_text_style(
            "Not connected",
            Point::new(8, 28),
            TEXT_XS!(),
            TextStyleBuilder::new()
                .alignment(Alignment::Left)
                .baseline(Baseline::Top)
                .build(),
        )
        .draw(display)
        .unwrap();
    }
}

fn draw_calibration_l(display: &mut impl DeviceDisplay, state: &CalibrationState) {
    draw_text_sm("Calibration", 17, 22, display);
    if state == &CalibrationState::Waiting {
        draw_double_press_lr_to_go_back(display);
    }
}

fn draw_calibration_r(display: &mut impl DeviceDisplay, state: &CalibrationState) {
    if state == &CalibrationState::Waiting {
        draw_calibrations_instructions(display);
    } else {
        draw_done(display);
    }
}

pub fn draw_settings_l(display: &mut impl DeviceDisplay, opt: &SettingsOption) {
    match opt {
        SettingsOption::WiFi(_) => draw_wifi_l(display),
        SettingsOption::Calibration(state) => draw_calibration_l(display, state),
    }
}

pub fn draw_settings_r(display: &mut impl DeviceDisplay, opt: &SettingsOption) {
    match opt {
        SettingsOption::WiFi(state) => draw_wifi_r(display, state),
        SettingsOption::Calibration(state) => draw_calibration_r(display, state),
    }
}
