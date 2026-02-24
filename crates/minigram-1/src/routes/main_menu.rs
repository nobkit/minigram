use core::sync::atomic::Ordering;

use super::Route;
use crate::display::{DISPLAY_CMD, DisplayCommand};
use crate::input::INPUT_CHANNEL;
use crate::input::{ButtonEvent, InputEvent};
use defmt::info;
use miniroute::{RouteHooks, route};
use portable_atomic::AtomicI32;

#[route(router = Route, hooks)]
pub enum MainMenuRoute {
    #[task(handle_input)]
    HandleInput,
}

impl RouteHooks for MainMenuRoute {
    async fn setup() {
        let selected_item = SELECTED_ITEM.load(Ordering::SeqCst);
        DISPLAY_CMD
            .send(DisplayCommand::MainMenu {
                selected: selected_item as u32,
            })
            .await;
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

static SELECTED_ITEM: AtomicI32 = AtomicI32::new(0);
#[embassy_executor::task]
async fn handle_input(route: MainMenuRoute) {
    route
        .task(async || match INPUT_CHANNEL.receive().await {
            InputEvent::Right(ButtonEvent::Click) => {
                SELECTED_ITEM
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value: i32| {
                        Option::Some((value + 1).rem_euclid(4))
                    })
                    .unwrap();
                DISPLAY_CMD
                    .send(DisplayCommand::MainMenu {
                        selected: SELECTED_ITEM.load(Ordering::SeqCst) as u32,
                    })
                    .await;
            }
            InputEvent::Left(ButtonEvent::Click) => {
                SELECTED_ITEM
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                        Option::Some((value - 1).rem_euclid(4))
                    })
                    .unwrap();
                DISPLAY_CMD
                    .send(DisplayCommand::MainMenu {
                        selected: SELECTED_ITEM.load(Ordering::SeqCst) as u32,
                    })
                    .await;
            }
            InputEvent::Both(ButtonEvent::Click) => match SELECTED_ITEM.load(Ordering::SeqCst) {
                0 => {
                    route.navigate(Route::Scale);
                }
                2 => {
                    route.navigate(Route::Settings);
                }
                _ => {
                    info!("Not yet implemented!");
                }
            },
            _ => {}
        })
        .run()
        .await;
}
