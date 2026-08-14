use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal, watch::Watch};
use embassy_time::Timer;
use minigram_1_ui::settings::{CalibrationState, SettingsOption, WiFiState};
use miniroute::{RouteHooks, route};

use crate::{
    display::{DISPLAY_CMD, DisplayCommand},
    input::{ButtonEvent, INPUT_CHANNEL, InputEvent},
    load_cell::{LOAD_CELL_COMMANDS, LOAD_CELL_RUNNING, LoadCellCommand},
    routes::Route,
};

#[route(router = Route, hooks)]
pub enum Settings {
    #[task(handle_input)]
    HandleInput,
    #[task(draw)]
    Draw,
    #[task(completed_timeout)]
    CompletedTimeout,
}

impl RouteHooks for Settings {
    async fn setup() {
        SETTINGS_STATE
            .sender()
            .send(SettingsOption::Calibration(CalibrationState::Browse));
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

pub static SETTINGS_STATE: Watch<CriticalSectionRawMutex, SettingsOption, 3> = Watch::new();

fn next_state(current: &SettingsOption) -> Option<SettingsOption> {
    match current {
        SettingsOption::Calibration(CalibrationState::Browse) => {
            Some(SettingsOption::Calibration(CalibrationState::Waiting))
        }
        SettingsOption::WiFi(WiFiState::Browse) => {
            Some(SettingsOption::WiFi(WiFiState::WaitForPairing))
        }
        _ => None,
    }
}

fn back_state(current: &SettingsOption) -> Option<SettingsOption> {
    match current {
        SettingsOption::Calibration(CalibrationState::Waiting) => {
            Some(SettingsOption::Calibration(CalibrationState::Browse))
        }
        SettingsOption::WiFi(_) => Some(SettingsOption::WiFi(WiFiState::Browse)),
        _ => None,
    }
}

fn is_browsing(current: &SettingsOption) -> bool {
    match current {
        SettingsOption::Calibration(CalibrationState::Browse) => true,
        SettingsOption::WiFi(WiFiState::Browse) => true,
        _ => false,
    }
}

pub static SETTINGS_OPTIONS: [SettingsOption; 2] = [
    SettingsOption::Calibration(CalibrationState::Browse),
    SettingsOption::WiFi(WiFiState::Browse),
];

#[embassy_executor::task]
async fn handle_input(route: Settings) {
    let mut settings_index: u8 = 0;
    route
        .task(async || match INPUT_CHANNEL.receive().await {
            InputEvent::Both(ButtonEvent::Click) => {
                let current = SETTINGS_STATE.try_get().unwrap();
                if let Some(next) = next_state(&current) {
                    SETTINGS_STATE.sender().send(next);
                }
            }
            InputEvent::Both(ButtonEvent::DoubleClick) => {
                let current = SETTINGS_STATE.try_get().unwrap();
                match back_state(&current) {
                    Some(prev) => {
                        SETTINGS_STATE.sender().send(prev);
                    }
                    None => {
                        route.navigate(Route::MainMenu);
                    }
                }
            }
            InputEvent::Both(ButtonEvent::Hold) => {
                let current = SETTINGS_STATE.try_get().unwrap();
                if current == SettingsOption::Calibration(CalibrationState::Waiting) {
                    LOAD_CELL_COMMANDS.send(LoadCellCommand::Calibrate).await;
                    SETTINGS_STATE
                        .sender()
                        .send(SettingsOption::Calibration(CalibrationState::Complete));
                    COMPLETED.signal(());
                }
            }
            InputEvent::Left(ButtonEvent::Click) => {
                let current = SETTINGS_STATE.try_get().unwrap();
                if !is_browsing(&current) {
                    return;
                }
                let len = SETTINGS_OPTIONS.len() as u8;
                settings_index = (settings_index + len - 1) % len;
                SETTINGS_STATE
                    .sender()
                    .send(SETTINGS_OPTIONS[settings_index as usize]);
            }
            InputEvent::Right(ButtonEvent::Click) => {
                let current = SETTINGS_STATE.try_get().unwrap();
                if !is_browsing(&current) {
                    return;
                }
                let len = SETTINGS_OPTIONS.len() as u8;
                settings_index = (settings_index + 1) % len;
                SETTINGS_STATE
                    .sender()
                    .send(SETTINGS_OPTIONS[settings_index as usize]);
            }
            _ => {}
        })
        .run()
        .await;
}

#[embassy_executor::task]
async fn draw(route: Settings) {
    let mut rx = SETTINGS_STATE.receiver().unwrap();
    route
        .task(async || {
            let opt = rx.changed().await;
            DISPLAY_CMD
                .send(DisplayCommand::Clear {
                    left: true,
                    right: true,
                })
                .await;
            DISPLAY_CMD.send(DisplayCommand::Settings(opt)).await;
        })
        .run()
        .await;
}

static COMPLETED: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[embassy_executor::task]
async fn completed_timeout(route: Settings) {
    route
        .task(async || {
            COMPLETED.wait().await;
            Timer::after_millis(1000).await;
            SETTINGS_STATE
                .sender()
                .send(SettingsOption::Calibration(CalibrationState::Browse));
        })
        .run()
        .await;
}
