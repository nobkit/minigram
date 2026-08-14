#![no_std]
use embedded_graphics::{pixelcolor::BinaryColor, prelude::DrawTarget};

pub mod icons;
pub mod main_menu;
pub mod scale;
pub mod settings;
pub mod settings_menu;
pub mod text;
pub mod utils;

pub trait DeviceDisplay: DrawTarget<Color = BinaryColor, Error: core::fmt::Debug> {}
impl<T: DrawTarget<Color = BinaryColor, Error: core::fmt::Debug>> DeviceDisplay for T {}
