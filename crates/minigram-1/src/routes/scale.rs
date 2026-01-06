use defmt::info;

use crate::{
    input::{ButtonEvent, InputEvent},
    load_cell::{LOAD_CELL_COMMANDS, LOAD_CELL_RUNNING, LoadCellCommand},
    routes::{Route, RouteName, SCALE, main_menu::MainMenu, run_on_route},
};

pub struct Scale;

impl Route for Scale {
    const ROUTE_NAME: RouteName = RouteName::Scale;

    async fn on_input(&self, input: InputEvent) {
        match input {
            InputEvent::Left(ButtonEvent::Click) => {
                info!("Current Route: Weighing Menu");
            }
            InputEvent::Right(ButtonEvent::Click) => {
                self.navigate::<MainMenu>().await;
            }
            InputEvent::Left(ButtonEvent::DoubleClick) => {
                LOAD_CELL_COMMANDS.send(LoadCellCommand::Calibrate).await;
            }
            _ => {}
        };
    }

    async fn on_enter(&self) {
        LOAD_CELL_RUNNING.sender().send(true);
    }

    async fn on_exit(&self) {
        LOAD_CELL_RUNNING.sender().send(false);
    }
}

#[embassy_executor::task]
pub async fn scale_route() {
    let route = SCALE.init(Scale {});
    run_on_route(route).await;
}
