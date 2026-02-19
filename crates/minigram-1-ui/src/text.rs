use embedded_graphics::pixelcolor::BinaryColor;
use u8g2_fonts::{
    U8g2TextStyle,
    fonts::{u8g2_font_logisoso16_tf, u8g2_font_logisoso38_tf},
};

pub static TEXT_L: U8g2TextStyle<BinaryColor> =
    U8g2TextStyle::new(u8g2_font_logisoso38_tf, BinaryColor::On);
pub static TEXT_S: U8g2TextStyle<BinaryColor> =
    U8g2TextStyle::new(u8g2_font_logisoso16_tf, BinaryColor::On);

#[macro_export]
macro_rules! TEXT_XS {
    () => {
        embedded_mogeefont::MogeeTextStyle::new(embedded_graphics::pixelcolor::BinaryColor::On)
    };
    ($color: expr) => {
        embedded_mogeefont::MogeeTextStyle::new($color)
    };
}
