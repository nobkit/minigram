pub mod main_menu;
pub mod scale;
pub mod settings;
pub mod settings_menu;

use crate::routes::{main_menu::MainMenuRoute, scale::ScaleRoute, settings::Settings};
use miniroute::router;

#[router]
pub enum Route {
    #[to(ScaleRoute)]
    Scale,
    #[to(MainMenuRoute)]
    MainMenu,
    #[to(Settings)]
    Settings,
    #[to(SettingsMenu)]
    SettingsMenu,
}
