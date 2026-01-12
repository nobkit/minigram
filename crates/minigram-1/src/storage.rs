use core::panic;

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, once_lock::OnceLock,
};
use esp_storage::FlashStorage;
use sequential_storage::{
    cache::KeyPointerCache,
    map::{Key, MapConfig, MapStorage, SerializationError},
};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StoreKey {
    ScaleCalibrationFactor = 1,
    WifiSSID = 2,
    WifiPassword = 3,
}

impl Key for StoreKey {
    fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        buffer[0] = *self as u8;
        Ok(1)
    }

    fn deserialize_from(buffer: &[u8]) -> Result<(Self, usize), SerializationError> {
        match buffer[0] {
            1 => Ok((StoreKey::ScaleCalibrationFactor, 1)),
            2 => Ok((StoreKey::WifiSSID, 2)),
            3 => Ok((StoreKey::WifiPassword, 3)),
            _ => Err(SerializationError::InvalidFormat),
        }
    }
}

pub static STORE: OnceLock<
    Mutex<
        CriticalSectionRawMutex,
        MapStorage<
            StoreKey,
            embassy_embedded_hal::adapter::BlockingAsync<FlashStorage<'_>>,
            KeyPointerCache<3, StoreKey, 8>,
        >,
    >,
> = OnceLock::new();

pub fn init_storage(flash: FlashStorage<'static>) {
    let flash = embassy_embedded_hal::adapter::BlockingAsync::new(flash);

    let mut storage: MapStorage<
        StoreKey,
        embassy_embedded_hal::adapter::BlockingAsync<FlashStorage<'_>>,
        KeyPointerCache<3, StoreKey, 8>,
    > = MapStorage::new(
        flash,
        MapConfig::new(0x9000..0xC000),
        KeyPointerCache::<3, StoreKey, 8>::new(),
    );

    match STORE.init(Mutex::new(storage)) {
        Err(_) => panic!("Failed to initialize flash storage."),
        _ => {}
    }
}
