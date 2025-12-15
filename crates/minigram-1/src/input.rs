#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_futures::select::{Either, select};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, Sender},
};
use embassy_time::{Duration, Timer, WithTimeout};
use esp_hal::gpio::{AnyPin, Input, InputConfig, Pull};

pub static LEFT_BUTTON_CHANNEL: Channel<CriticalSectionRawMutex, ButtonEvent, 10> = Channel::new();
pub static RIGHT_BUTTON_CHANNEL: Channel<CriticalSectionRawMutex, ButtonEvent, 10> = Channel::new();

pub enum ButtonId {
    Left = -1,
    Right = 1,
}

#[derive(Clone, Copy, PartialEq, defmt::Format)]
pub enum ButtonEvent {
    Click,
    DoubleClick,
    Hold,
}

#[embassy_executor::task(pool_size = 2)]
pub async fn handle_button(
    pin: AnyPin<'static>,
    sender: Sender<'static, CriticalSectionRawMutex, ButtonEvent, 10>,
) {
    const HOLD_TIMEOUT: Duration = Duration::from_secs(1);
    const DOUBLE_CLICK_TIMEOUT: Duration = Duration::from_millis(100);

    let mut button = Input::new(pin, InputConfig::default().with_pull(Pull::Up));

    loop {
        Timer::after_millis(20).await;
        button.wait_for_low().await;

        Timer::after_millis(20).await;
        match button.wait_for_high().with_timeout(HOLD_TIMEOUT).await {
            Err(_) => {
                sender.send(ButtonEvent::Hold).await;

                button.wait_for_high().await;
                continue;
            }
            _ => (),
        }

        Timer::after_millis(20).await;
        match button
            .wait_for_low()
            .with_timeout(DOUBLE_CLICK_TIMEOUT)
            .await
        {
            Ok(_) => {
                Timer::after_millis(20).await;
                button.wait_for_high().await;

                sender.send(ButtonEvent::DoubleClick).await
            }
            Err(_) => sender.send(ButtonEvent::Click).await,
        }
    }
}

#[derive(Clone, Copy, PartialEq, defmt::Format)]
enum InputEvent {
    Left(ButtonEvent),
    Right(ButtonEvent),
    Both(ButtonEvent),
}

#[embassy_executor::task]
pub async fn handle_gestures() {
    const BOTH_PRESSED_TIMEOUT: Duration = Duration::from_millis(50);

    loop {
        let (button_id, button_event, other_channel) = match select(
            LEFT_BUTTON_CHANNEL.receive(),
            RIGHT_BUTTON_CHANNEL.receive(),
        )
        .await
        {
            Either::First(be) => (ButtonId::Left, be, &RIGHT_BUTTON_CHANNEL),
            Either::Second(be) => (ButtonId::Right, be, &LEFT_BUTTON_CHANNEL),
        };

        match other_channel
            .receive()
            .with_timeout(BOTH_PRESSED_TIMEOUT)
            .await
        {
            Ok(be) if be == button_event => {
                info!("{}", InputEvent::Both(button_event));
            }
            result => match button_id {
                ButtonId::Left => {
                    info!("{}", InputEvent::Left(button_event));
                    if let Some(other_button_event) = result.ok() {
                        info!("{}", InputEvent::Right(other_button_event));
                    }
                }
                ButtonId::Right => {
                    info!("{}", InputEvent::Right(button_event));
                    if let Some(other_button_event) = result.ok() {
                        info!("{}", InputEvent::Left(other_button_event));
                    }
                }
            },
        };
    }
}
