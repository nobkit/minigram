use super::Route;
use crate::input::INPUT_CHANNEL;
use crate::input::{ButtonEvent, InputEvent};
use defmt::info;
use miniroute::route;

#[route(router = Route)]
pub enum MainMenuRoute {
    #[task(handle_input)]
    HandleInput,
}

#[embassy_executor::task]
async fn handle_input(route: MainMenuRoute) {}
