pub mod main_menu;
pub mod scale;

use crate::routes::{main_menu::MainMenuRoute, scale::ScaleRoute};
use miniroute::router;

#[router]
pub enum Route {
    #[to(ScaleRoute)]
    Scale,
    #[to(MainMenuRoute)]
    MainMenu,
}
