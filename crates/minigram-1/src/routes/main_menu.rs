use defmt::info;

use crate::{
    input::{ButtonEvent, InputEvent},
    routes::{MAIN_MENU, Route, RouteName, run_on_route, scale::Scale},
};

pub struct MainMenu;

impl Route for MainMenu {
    const ROUTE_NAME: RouteName = RouteName::MainMenu;

    async fn on_input(&self, input: InputEvent) {
        match input {
            InputEvent::Left(ButtonEvent::Click) => {
                info!("Current Route: Main Menu");
            }
            InputEvent::Right(ButtonEvent::Click) => {
                self.navigate::<Scale>().await;
            }
            _ => {}
        };
    }
}

#[embassy_executor::task]
pub async fn main_menu_route() {
    let route = MAIN_MENU.init(MainMenu {});
    run_on_route(route).await;
}
