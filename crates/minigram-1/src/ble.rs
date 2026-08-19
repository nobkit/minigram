//! BLE peripheral used to provision the scale.
//!
//! The Wi-Fi credentials are only reachable over an authenticated link. Pairing
//! uses LE Secure Connections passkey entry: the scale renders a six digit code
//! on the right-hand display and the phone has to type it back, so whoever
//! writes the credentials is standing in front of the device.
//!
//! Consuming the credentials is not wired up yet; writes are only logged.

// `gatt_service` borrows the default characteristic values where it need not,
// and the lint lands on our field spans.
#![allow(clippy::needless_borrows_for_generic_args)]

use defmt::{info, warn};

use embassy_futures::join::join;

use esp_hal::{peripherals::BT, rng::Trng};
use esp_radio::ble::controller::BleConnector;

use minigram_1_ui::settings::WiFiState;

use trouble_host::prelude::*;

use crate::routes::{Route, wifi::WIFI_STATE};

const CONNECTIONS_MAX: usize = 1;

const L2CAP_CHANNELS_MAX: usize = 1;

/// Number of HCI commands that may be in flight towards the controller.
const HCI_SLOTS: usize = 20;

/// Also the advertised local name, so it has to stay under 22 bytes.
const DEVICE_NAME: &str = "minigram";

#[gatt_server]
struct Server {
    wifi_service: WiFiService,
}

#[gatt_service(uuid = "890b31b6-6f34-412b-9027-e8652de2fa84")]
struct WiFiService {
    #[characteristic(
        uuid = "790b31b6-6f34-412b-9027-e8652de2fa84",
        read,
        write,
        permissions(authenticated)
    )]
    ssid: heapless::Vec<u8, 32>,
    /// Write only on purpose: the scale never hands the passphrase back out.
    #[characteristic(
        uuid = "790b01b6-6f34-412b-9027-e8652de2fa84",
        write,
        permissions(authenticated)
    )]
    passkey: heapless::Vec<u8, 64>,
}

/// Advertise and serve the provisioning service for as long as the scale is on.
///
/// Requires an [`esp_hal::rng::TrngSource`] to have been enabled by the caller:
/// the security manager refuses to run off a non-cryptographic generator.
#[embassy_executor::task]
pub async fn ble(bt: BT<'static>) {
    let mut trng = match Trng::try_new() {
        Ok(trng) => trng,
        Err(_) => {
            warn!("BLE disabled: no TrngSource is active");
            return;
        }
    };

    let connector = match BleConnector::new(bt, esp_radio::ble::Config::default()) {
        Ok(connector) => connector,
        Err(e) => {
            warn!("BLE disabled: controller init failed: {}", e);
            return;
        }
    };

    let controller: ExternalController<_, HCI_SLOTS> = ExternalController::new(connector);

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();

    let mut address = [0u8; 6];
    trng.read(&mut address);
    // The two most significant bits mark this as a static random address.
    address[5] |= 0xC0;

    let stack = trouble_host::new(controller, &mut resources)
        .set_random_address(Address::random(address))
        .set_random_generator_seed(&mut trng);

    // The scale can show a passkey but cannot take input over BLE, which pins
    // pairing to passkey entry with the phone as the side doing the typing.
    stack.set_io_capabilities(IoCapabilities::DisplayOnly);

    let Host {
        mut peripheral,
        mut runner,
        ..
    } = stack.build();

    let server = match Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: DEVICE_NAME,
        appearance: &appearance::WEIGHT_SCALE,
    })) {
        Ok(server) => server,
        Err(e) => {
            warn!("BLE disabled: GATT server setup failed: {}", e);
            return;
        }
    };

    let mut adv_data = [0u8; 31];
    let adv_len = match AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::CompleteLocalName(DEVICE_NAME.as_bytes()),
        ],
        &mut adv_data,
    ) {
        Ok(len) => len,
        Err(e) => {
            warn!("BLE disabled: the advertisement does not fit: {}", e);
            return;
        }
    };

    let host = async {
        loop {
            if let Err(e) = runner.run().await {
                warn!("BLE host stopped, restarting: {}", e);
            }
        }
    };

    let sessions = async {
        loop {
            let advertiser = match peripheral
                .advertise(
                    &AdvertisementParameters::default(),
                    Advertisement::ConnectableScannableUndirected {
                        adv_data: &adv_data[..adv_len],
                        scan_data: &[],
                    },
                )
                .await
            {
                Ok(advertiser) => advertiser,
                Err(e) => {
                    warn!("Advertising failed: {}", e);
                    continue;
                }
            };

            let connection = match advertiser.accept().await {
                Ok(connection) => connection,
                Err(e) => {
                    warn!("Accepting a connection failed: {}", e);
                    continue;
                }
            };

            match connection.with_attribute_server(&server) {
                Ok(connection) => provision(&connection, &server).await,
                Err(e) => warn!("Attaching the attribute server failed: {}", e),
            }
        }
    };

    join(host, sessions).await;
}

/// Drive a single provisioning session until the phone goes away.
async fn provision(connection: &GattConnection<'_, '_, DefaultPacketPool>, server: &Server<'_>) {
    // Every characteristic sits behind `AuthenticationRequired`, so ask to pair
    // up front rather than waiting for the first rejected read.
    if let Err(e) = connection.raw().request_security() {
        warn!("Could not request pairing: {}", e);
    }

    loop {
        match connection.next().await {
            GattConnectionEvent::Disconnected { reason } => {
                info!("Provisioning client disconnected: {}", reason);
                stop_showing_passkey();
                break;
            }
            GattConnectionEvent::PassKeyDisplay(pass_key) => {
                info!("Showing the pairing pass key");
                show_passkey(pass_key.value());
            }
            // `DisplayOnly` should never negotiate either of these, and there is
            // no way to answer them, so refuse instead of silently settling for
            // an unauthenticated pairing.
            GattConnectionEvent::PassKeyConfirm(_) | GattConnectionEvent::PassKeyInput => {
                warn!("Peer asked for a pairing method the scale cannot perform");
                let _ = connection.pass_key_cancel();
            }
            GattConnectionEvent::PairingComplete { security_level, .. } => {
                info!("Pairing complete at {}", security_level);
                stop_showing_passkey();
            }
            GattConnectionEvent::PairingFailed(e) => {
                warn!("Pairing failed: {}", e);
                stop_showing_passkey();
            }
            GattConnectionEvent::Gatt { event } => {
                match &event {
                    GattEvent::Write(write) => {
                        if write.handle() == server.wifi_service.ssid.handle {
                            info!("SSID written ({} bytes)", write.data().len());
                        } else if write.handle() == server.wifi_service.passkey.handle {
                            info!("Passphrase written ({} bytes)", write.data().len());
                        }
                    }
                    GattEvent::NotAllowed(_) => {
                        warn!("Refused the Wi-Fi credentials: the link is not authenticated");
                    }
                    _ => {}
                }

                match event.accept() {
                    Ok(reply) => reply.send().await,
                    Err(e) => warn!("Could not reply to a GATT request: {}", e),
                }
            }
            _ => {}
        }
    }
}

/// Put the pass key on screen, pulling the user to the route that draws it.
fn show_passkey(pass_key: u32) {
    WIFI_STATE.sender().send(WiFiState::Pairing(pass_key));
    Route::start(Route::WiFi);
}

/// Take the pass key back off screen once it is no longer usable.
fn stop_showing_passkey() {
    if let Some(WiFiState::Pairing(_)) = WIFI_STATE.try_get() {
        WIFI_STATE.sender().send(WiFiState::NotConnected);
    }
}
