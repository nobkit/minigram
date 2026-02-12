use super::Route;
use crate::{
    display::{DISPLAY_CMD, DisplayCommand},
    input::{ButtonEvent, INPUT_CHANNEL, InputEvent},
};
use core::sync::atomic::{AtomicBool, Ordering};
use defmt::info;
use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal, watch::Watch};
use embassy_time::{Duration, Instant, Timer};
use miniroute::{RouteHooks, route};
use portable_atomic::AtomicU64;

#[route(router = Route)]
pub enum ScaleRoute {
    #[task(handle_input)]
    HandleInput,
    #[task(timer)]
    Timer,
    #[task(logger)]
    Logger,
}

#[derive(Clone, Copy)]
enum TimerCmd {
    Toggle,
    Reset,
}

static TIMER_CMD: Signal<CriticalSectionRawMutex, TimerCmd> = Signal::new();
static SECONDS_ELAPSED: Watch<CriticalSectionRawMutex, u64, 2> = Watch::new();

#[embassy_executor::task]
async fn handle_input(route: ScaleRoute) {
    route
        .task(async || match INPUT_CHANNEL.receive().await {
            InputEvent::Left(ButtonEvent::Click) => {
                info!("Current Route: Weighing Menu");
            }
            InputEvent::Right(ButtonEvent::Click) => {
                TIMER_CMD.signal(TimerCmd::Toggle);
            }
            InputEvent::Right(ButtonEvent::Hold) => {
                TIMER_CMD.signal(TimerCmd::Reset);
            }
            _ => {}
        })
        .run()
        .await;
}

#[embassy_executor::task]
async fn timer(route: ScaleRoute) {
    let elapsed = AtomicU64::new(0);
    let running = AtomicBool::new(false);
    let mut anchor = Instant::now();
    let tx = SECONDS_ELAPSED.sender();

    route
        .task(async || {
            if running.load(Ordering::Relaxed) {
                let e = elapsed.load(Ordering::Relaxed);
                let next_tick = anchor + Duration::from_secs(e + 1);
                match select(Timer::at(next_tick), TIMER_CMD.wait()).await {
                    Either::First(_) => {
                        let new_e = e + 1;
                        elapsed.store(new_e, Ordering::Relaxed);
                        tx.send(new_e);
                    }
                    Either::Second(cmd) => match cmd {
                        TimerCmd::Toggle => running.store(false, Ordering::Relaxed),
                        TimerCmd::Reset => {
                            running.store(false, Ordering::Relaxed);
                            elapsed.store(0, Ordering::Relaxed);
                            tx.send(0);
                        }
                    },
                }
            } else {
                match TIMER_CMD.wait().await {
                    TimerCmd::Toggle => {
                        running.store(true, Ordering::Relaxed);
                        let e = elapsed.load(Ordering::Relaxed);
                        anchor = Instant::now() - Duration::from_secs(e);
                    }
                    TimerCmd::Reset => {
                        elapsed.store(0, Ordering::Relaxed);
                        tx.send(0);
                    }
                }
            }
        })
        .setup(async || {
            elapsed.store(0, Ordering::Relaxed);
            running.store(false, Ordering::Relaxed);
            TIMER_CMD.reset();
            tx.send(0);
        })
        .cleanup(async || {
            running.store(false, Ordering::Relaxed);
            TIMER_CMD.reset();
        })
        .run()
        .await;
}

#[embassy_executor::task]
async fn logger(route: ScaleRoute) {
    let mut rx = SECONDS_ELAPSED.receiver().unwrap();
    route
        .task(async || {
            let secs = rx.changed().await;
            // draw_timer();
            // info!("{}", secs);
            DISPLAY_CMD.signal(DisplayCommand::Timer {
                time: secs,
                paused: false,
            })
        })
        .run()
        .await;
}
