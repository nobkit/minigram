use defmt::info;
use embassy_futures::select::select;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    channel::{Channel, Receiver, TryReceiveError},
    mutex::Mutex,
    signal::Signal,
    watch::Watch,
};
use embassy_time::Timer;
use esp_hal::{
    delay::Delay,
    gpio::{AnyPin, Input, InputConfig, Level, Output, OutputConfig, Pull},
};
use loadcell::{LoadCell, hx711::HX711};
use static_cell::StaticCell;

use crate::{
    input::{ButtonEvent, InputEvent},
    routes::{RouteName, run_on_route},
};

pub static WEIGHT: Signal<CriticalSectionRawMutex, f64> = Signal::new();

pub static LOAD_CELL_RUNNING: Watch<CriticalSectionRawMutex, bool, 1> = Watch::new();

static LOAD_CELL: StaticCell<Mutex<NoopRawMutex, HX711<Output<'_>, Input<'_>, Delay>>> =
    StaticCell::new();

pub enum LoadCellCommand {
    Tare,
    Calibrate,
}

pub static LOAD_CELL_COMMANDS: Channel<CriticalSectionRawMutex, LoadCellCommand, 8> =
    Channel::new();

#[embassy_executor::task]
pub async fn load_cell(sck: AnyPin<'static>, dt: AnyPin<'static>) {
    let hx711_sck = Output::new(sck, Level::Low, OutputConfig::default());
    let hx711_dt = Input::new(dt, InputConfig::default().with_pull(Pull::None));
    let delay = Delay::new();

    let mut load_sensor = HX711::new(hx711_sck, hx711_dt, delay);

    Timer::after_millis(2000).await;

    load_sensor.tare(32);

    load_sensor.set_scale(1.0);

    let load_sensor_mutex = LOAD_CELL.init(Mutex::new(load_sensor));

    if let Some(mut running) = LOAD_CELL_RUNNING.receiver() {
        loop {
            running.get_and(|n| *n).await;

            match LOAD_CELL_COMMANDS.try_receive() {
                Ok(LoadCellCommand::Tare) => {
                    let mut load = load_sensor_mutex.lock().await;
                    load.tare(32);
                }
                Ok(LoadCellCommand::Calibrate) => {
                    let mut load = load_sensor_mutex.lock().await;
                    if let Ok(weight) = load.read_scaled() {
                        WEIGHT.signal(weight as f64);
                        load.set_scale(100000.0 / weight);
                    }
                }
                Err(TryReceiveError::Empty) => {
                    let mut load = load_sensor_mutex.lock().await;
                    if let Ok(weight) = load.read_scaled() {
                        WEIGHT.signal(weight as f64);
                    }
                }
            };

            Timer::after_millis(100).await;
        }
    };
}
