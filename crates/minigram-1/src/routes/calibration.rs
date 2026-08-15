use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal, watch::Watch};
use embassy_time::Timer;
use minigram_1_ui::settings::{CalibrationState, SettingsOption};
use miniroute::{RouteHooks, route};

use crate::{
    display::{DISPLAY_CMD, DisplayCommand},
    input::{ButtonEvent, INPUT_CHANNEL, InputEvent},
    load_cell::{LOAD_CELL_COMMANDS, LOAD_CELL_RUNNING, LoadCellCommand},
    routes::Route,
};

#[route(router = Route, hooks)]
pub enum CalibrationRoute {
    #[task(handle_input)]
    HandleInput,
    #[task(draw)]
    Draw,
    #[task(completed_timeout)]
    CompletedTimeout,
}

static CALIBRATION_STATE: Watch<CriticalSectionRawMutex, CalibrationState, 1> = Watch::new();

impl RouteHooks for CalibrationRoute {
    async fn setup() {
        CALIBRATION_STATE.sender().send(CalibrationState::Waiting);
        LOAD_CELL_RUNNING.sender().send(true);
    }

    async fn cleanup() {
        DISPLAY_CMD
            .send(DisplayCommand::Clear {
                left: true,
                right: true,
            })
            .await;
        LOAD_CELL_RUNNING.sender().send(false);
    }
}

#[embassy_executor::task]
async fn handle_input(route: CalibrationRoute) {
    route
        .task(async || match INPUT_CHANNEL.receive().await {
            InputEvent::Both(ButtonEvent::Hold) => {
                if CALIBRATION_STATE.try_get() == Some(CalibrationState::Waiting) {
                    LOAD_CELL_COMMANDS.send(LoadCellCommand::Calibrate).await;
                    CALIBRATION_STATE.sender().send(CalibrationState::Complete);
                    COMPLETED.signal(());
                }
            }
            InputEvent::Both(ButtonEvent::DoubleClick) => {
                route.navigate(Route::SettingsMenu);
            }
            _ => {}
        })
        .run()
        .await;
}

#[embassy_executor::task]
async fn draw(route: CalibrationRoute) {
    let mut rx = CALIBRATION_STATE.receiver().unwrap();
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
                .send(DisplayCommand::Settings(SettingsOption::Calibration(state)))
                .await;
        })
        .run()
        .await;
}

static COMPLETED: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[embassy_executor::task]
async fn completed_timeout(route: CalibrationRoute) {
    route
        .task(async || {
            COMPLETED.wait().await;
            Timer::after_millis(1000).await;
            route.navigate(Route::SettingsMenu);
        })
        .setup(async || COMPLETED.reset())
        .run()
        .await;
}
