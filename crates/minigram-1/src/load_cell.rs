use defmt::info;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    channel::{Channel, TryReceiveError},
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

use crate::storage::{STORE, StoreKey};

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

    let mut data_buffer = [0u8; 64];
    let mut store = STORE.get().await.lock().await;
    if let Ok(Some(factor)) = store
        .fetch_item::<f32>(&mut data_buffer, &StoreKey::ScaleCalibrationFactor)
        .await
    {
        info!("Scale calibration loaded from flash: {}", &factor);
        load_sensor.set_scale(factor);
    } else {
        info!("Scale calibration not found. Defaulting to 1.0");
        load_sensor.set_scale(1.0);
    }
    drop(store);

    let load_sensor_mutex = LOAD_CELL.init(Mutex::new(load_sensor));

    if let Some(mut running) = LOAD_CELL_RUNNING.receiver() {
        let mut buffer = [0f32; 5];
        let mut index = 0usize;
        let mut filled = false;

        loop {
            running.get_and(|n| *n).await;

            match LOAD_CELL_COMMANDS.try_receive() {
                Ok(LoadCellCommand::Tare) => {
                    let mut load = load_sensor_mutex.lock().await;
                    load.tare(32);
                    buffer = [0f32; 5];
                    index = 0;
                    filled = false;
                }
                Ok(LoadCellCommand::Calibrate) => {
                    let mut load = load_sensor_mutex.lock().await;
                    let mut store = STORE.get().await.lock().await;
                    load.set_scale(1.0);
                    if let Ok(weight) = load.read_scaled() {
                        WEIGHT.signal(weight as f64);
                        let scale = 100000.0 / weight;
                        match store
                            .store_item(&mut data_buffer, &StoreKey::ScaleCalibrationFactor, &scale)
                            .await
                        {
                            Ok(_) => {
                                info!("Calibration stored into flash: {}", &scale);
                            }
                            Err(e) => {
                                info!("Error storing calibration into flash: {}", &e)
                            }
                        }
                        load.set_scale(scale);
                        buffer = [0f32; 5];
                        index = 0;
                        filled = false;
                    }
                }
                Err(TryReceiveError::Empty) => {
                    let mut load = load_sensor_mutex.lock().await;
                    if let Ok(weight) = load.read_scaled() {
                        buffer[index] = weight;
                        index = (index + 1) % 5;
                        if index == 0 {
                            filled = true;
                        }

                        let count = if filled { 5 } else { index.max(1) };
                        let smoothed = buffer[..count].iter().sum::<f32>() / count as f32;
                        info!("{}", smoothed);
                        WEIGHT.signal(smoothed as f64);
                    }
                }
            };

            Timer::after_millis(100).await;
        }
    };
}
