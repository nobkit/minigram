use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};

use crate::DeviceDisplay;

pub const HIGHLIGHT_SIZE: Size = Size::new(56, 64);
pub const LEFT_ORIGIN: Point = Point::new(0, 0);
pub const RIGHT_ORIGIN: Point = Point::new(72, 0);

pub fn draw_highlight(display: &mut impl DeviceDisplay, origin: Point) {
    Rectangle::new(origin, HIGHLIGHT_SIZE)
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(display)
        .unwrap();
}
