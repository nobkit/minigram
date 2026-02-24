pub mod main_menu;
pub mod scale;
pub mod settings;

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
}
