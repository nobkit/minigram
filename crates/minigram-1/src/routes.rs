pub mod calibration;
pub mod main_menu;
pub mod scale;
pub mod settings_menu;
pub mod wifi;

use crate::routes::{
    calibration::CalibrationRoute, main_menu::MainMenuRoute, scale::ScaleRoute,
    settings_menu::SettingsMenuRoute, wifi::WiFiRoute,
};
use miniroute::router;

#[router]
pub enum Route {
    #[to(ScaleRoute)]
    Scale,
    #[to(MainMenuRoute)]
    MainMenu,
    #[to(SettingsMenuRoute)]
    SettingsMenu,
    #[to(WiFiRoute)]
    WiFi,
    #[to(CalibrationRoute)]
    Calibration,
}
