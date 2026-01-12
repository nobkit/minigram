use core::{
    cell::Cell,
    future::{self},
};
use embassy_futures::select::select;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use static_cell::StaticCell;

use crate::input::{INPUT_CHANNEL, InputEvent};

pub mod main_menu;
pub mod scale;

#[derive(Clone, Copy, PartialEq, defmt::Format)]
pub enum RouteName {
    MainMenu,
    Scale,
}

static MAIN_MENU: StaticCell<main_menu::MainMenu> = StaticCell::new();
static SCALE: StaticCell<scale::Scale> = StaticCell::new();

#[allow(async_fn_in_trait)]
pub trait Route {
    const ROUTE_NAME: RouteName;
    async fn on_enter(&self) {}
    async fn on_input(&self, _input: InputEvent) {}
    async fn on_tick(&self) {
        future::pending::<()>().await;
    }
    async fn on_exit(&self) {}
    async fn navigate<R: Route>(&self) {
        self.on_exit().await;
        ROUTE.sender().send(R::ROUTE_NAME);
    }
}

pub static ROUTE: Watch<CriticalSectionRawMutex, RouteName, 3> = Watch::new();

pub async fn run_on_route<R: Route>(r: &R) {
    let input_receiver = INPUT_CHANNEL.receiver();
    if let Some(mut rcv) = ROUTE.receiver() {
        let last_value = Cell::new(None);
        let run_on_enter = Cell::new(false);
        loop {
            rcv.get_and(|n| {
                match (last_value.get(), n) {
                    (Some(a), &b) if a == b => {}
                    (_, &b) if b == R::ROUTE_NAME => {
                        run_on_enter.set(true);
                    }
                    _ => {}
                }
                last_value.set(Some(*n));
                *n == R::ROUTE_NAME
            })
            .await;

            if run_on_enter.get() {
                r.on_enter().await;
                run_on_enter.set(false);
            }

            select(
                async {
                    let input_event = input_receiver.receive().await;
                    r.on_input(input_event).await;
                },
                r.on_tick(),
            )
            .await;
        }
    };
}
