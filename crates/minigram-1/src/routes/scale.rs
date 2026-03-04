use super::Route;
use crate::{
    display::{DISPLAY_CMD, DisplayCommand},
    input::{ButtonEvent, INPUT_CHANNEL, InputEvent},
    load_cell::{LOAD_CELL_COMMANDS, LOAD_CELL_RUNNING, LoadCellCommand, WEIGHT},
};
use core::sync::atomic::{AtomicBool, Ordering};
use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal, watch::Watch};
use embassy_time::{Duration, Instant, Timer};
use miniroute::{RouteHooks, route};
use portable_atomic::AtomicU64;

#[route(router = Route, hooks)]
pub enum ScaleRoute {
    #[task(handle_input)]
    HandleInput,
    #[task(timer)]
    Timer,
    #[task(draw_timer)]
    DrawTimer,
    #[task(draw_weight)]
    DrawScale,
    #[task(draw_paused)]
    DrawPaused,
}

impl RouteHooks for ScaleRoute {
    async fn setup() {
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

#[derive(Clone, Copy)]
enum TimerCmd {
    Toggle,
    Reset,
}

static TIMER_CMD: Signal<CriticalSectionRawMutex, TimerCmd> = Signal::new();
static SECONDS_ELAPSED: Watch<CriticalSectionRawMutex, u64, 2> = Watch::new();
static RUNNING: Signal<CriticalSectionRawMutex, bool> = Signal::new();

#[embassy_executor::task]
async fn handle_input(route: ScaleRoute) {
    route
        .task(async || match INPUT_CHANNEL.receive().await {
            InputEvent::Left(ButtonEvent::Click) => {
                LOAD_CELL_COMMANDS.send(LoadCellCommand::Tare).await;
            }
            InputEvent::Right(ButtonEvent::Click) => {
                TIMER_CMD.signal(TimerCmd::Toggle);
            }
            InputEvent::Right(ButtonEvent::Hold) => {
                TIMER_CMD.signal(TimerCmd::Reset);
            }
            InputEvent::Both(ButtonEvent::Hold) => {
                route.navigate(Route::MainMenu);
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
                        TimerCmd::Toggle => {
                            running.store(false, Ordering::SeqCst);
                            RUNNING.signal(false);
                        }
                        TimerCmd::Reset => {
                            running.store(false, Ordering::SeqCst);
                            RUNNING.signal(false);
                            elapsed.store(0, Ordering::Relaxed);
                            tx.send(0);
                        }
                    },
                }
            } else {
                match TIMER_CMD.wait().await {
                    TimerCmd::Toggle => {
                        running.store(true, Ordering::SeqCst);
                        RUNNING.signal(true);
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
            running.store(false, Ordering::SeqCst);
            RUNNING.signal(false);
            TIMER_CMD.reset();
            tx.send(0);
        })
        .cleanup(async || {
            running.store(false, Ordering::SeqCst);
            RUNNING.signal(false);
            TIMER_CMD.reset();
        })
        .run()
        .await;
}

#[embassy_executor::task]
async fn draw_timer(route: ScaleRoute) {
    let mut rx = SECONDS_ELAPSED.receiver().unwrap();
    route
        .task(async || {
            let secs = rx.changed().await;
            // draw_timer();
            // info!("{}", secs);
            DISPLAY_CMD.send(DisplayCommand::Timer { time: secs }).await;
        })
        .setup(async || {
            DISPLAY_CMD.send(DisplayCommand::Timer { time: 0 }).await;
            DISPLAY_CMD
                .send(DisplayCommand::Paused { paused: true })
                .await;
        })
        .run()
        .await;
}

#[embassy_executor::task]
async fn draw_weight(route: ScaleRoute) {
    route
        .task(async || {
            let weight = WEIGHT.wait().await;
            DISPLAY_CMD
                .send(DisplayCommand::Scale {
                    weight: (weight as i64 / 100),
                })
                .await
        })
        .run()
        .await;
}

#[embassy_executor::task]
async fn draw_paused(route: ScaleRoute) {
    route
        .task(async || {
            DISPLAY_CMD
                .send(DisplayCommand::Paused {
                    paused: !RUNNING.wait().await,
                })
                .await;
        })
        .run()
        .await;
}
