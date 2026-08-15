use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use minigram_1_ui::settings::{SettingsOption, WiFiState};
use miniroute::{RouteHooks, route};

use crate::{
    display::{DISPLAY_CMD, DisplayCommand},
    input::{ButtonEvent, INPUT_CHANNEL, InputEvent},
    routes::Route,
};

#[route(router = Route, hooks)]
pub enum WiFiRoute {
    #[task(handle_input)]
    HandleInput,
    #[task(draw)]
    Draw,
}

pub static WIFI_STATE: Watch<CriticalSectionRawMutex, WiFiState, 1> = Watch::new();

impl RouteHooks for WiFiRoute {
    async fn setup() {
        let state = WIFI_STATE.try_get().unwrap_or(WiFiState::NotConnected);
        WIFI_STATE.sender().send(state);
    }

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
async fn handle_input(route: WiFiRoute) {
    route
        .task(async || {
            if let InputEvent::Both(ButtonEvent::DoubleClick) = INPUT_CHANNEL.receive().await {
                route.navigate(Route::SettingsMenu);
            }
        })
        .run()
        .await;
}

#[embassy_executor::task]
async fn draw(route: WiFiRoute) {
    let mut rx = WIFI_STATE.receiver().unwrap();
    route
        .task(async || {
            let state = rx.changed().await;
            DISPLAY_CMD
                .send(DisplayCommand::Clear {
                    left: true,
                    right: true,
                })
                .await;
            DISPLAY_CMD
                .send(DisplayCommand::Settings(SettingsOption::WiFi(state)))
                .await;
        })
        .run()
        .await;
}
