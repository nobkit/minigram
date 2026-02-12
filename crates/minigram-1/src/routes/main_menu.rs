use super::Route;
use crate::display::{DISPLAY_CMD, DisplayCommand};
use crate::input::INPUT_CHANNEL;
use crate::input::{ButtonEvent, InputEvent};
use defmt::info;
use miniroute::{RouteHooks, route};

#[route(router = Route, hooks)]
pub enum MainMenuRoute {
    #[task(handle_input)]
    HandleInput,
    #[task(draw_main_menu)]
    DrawMainMenu,
}

impl RouteHooks for MainMenuRoute {
    async fn cleanup() {
        DISPLAY_CMD
            .send(DisplayCommand::Clear {
                left: true,
                right: true,
            })
            .await;
    }
}

#[embassy_executor::task]
async fn handle_input(route: MainMenuRoute) {
    route
        .task(async || match INPUT_CHANNEL.receive().await {
            InputEvent::Right(ButtonEvent::Hold) => {
                route.navigate(Route::Scale);
            }
            InputEvent::Right(ButtonEvent::Click) => {
                info!("Current Route: Main Menu");
            }
            _ => {}
        })
        .run()
        .await;
}

#[embassy_executor::task]
async fn draw_main_menu(route: MainMenuRoute) {
    route
        .task(async || {})
        .setup(async || {
            DISPLAY_CMD.send(DisplayCommand::MainMenu).await;
        })
        .run()
        .await;
}
