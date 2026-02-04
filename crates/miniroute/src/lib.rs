#![no_std]
#![doc = include_str!("../README.md")]

/// Lifecycle command sent to tasks via the LIFECYCLE Watch.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RouteCommand {
    Start,
    Stop,
}

/// Trait for route lifecycle hooks.
///
/// Use the `hooks` flag in `#[route(...)]` to opt-in to a custom implementation.
#[allow(async_fn_in_trait)]
pub trait RouteHooks {
    /// Called when the route becomes active, before tasks receive Start.
    async fn setup() {}
    /// Called after all tasks have acknowledged Stop.
    async fn cleanup() {}
}

/// Trait for optional async callbacks in the task builder.
///
/// Implemented by `AsyncFnMut() -> ()` closures (blanket) and [`Noop`].
#[allow(async_fn_in_trait)]
pub trait Callback {
    /// Execute the callback.
    async fn call(&mut self);
}

/// No-op callback, used as the default for optional builder steps.
pub struct Noop;

impl Callback for Noop {
    async fn call(&mut self) {}
}

impl<F: AsyncFnMut() -> ()> Callback for F {
    async fn call(&mut self) {
        self().await;
    }
}

pub use miniroute_macro::{route, router};

#[doc(hidden)]
pub mod __private {
    pub use embassy_executor::Spawner;
    pub use embassy_futures::select::{Either, select};
    pub use embassy_futures::yield_now;
    pub use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    pub use embassy_sync::channel::Channel;
    pub use embassy_sync::pubsub::{PubSubChannel, WaitResult};
    pub use embassy_sync::signal::Signal;
    pub use embassy_sync::watch::Watch;
    pub use embassy_time::{Duration, WithTimeout};
    pub use heapless::Vec;

    pub use crate::{Callback, Noop, RouteCommand, RouteHooks};

    #[cfg(feature = "defmt")]
    pub use defmt;

    #[cfg(feature = "defmt")]
    #[inline]
    pub fn log_unknown_task_ack() {
        defmt::error!("Unknown task acknowledged stop");
    }

    #[cfg(not(feature = "defmt"))]
    #[inline]
    pub fn log_unknown_task_ack() {}

    #[cfg(feature = "defmt")]
    #[inline]
    pub fn log_tasks_timeout() {
        defmt::error!("Tasks failed to stop in time");
    }

    #[cfg(not(feature = "defmt"))]
    #[inline]
    pub fn log_tasks_timeout() {}

    #[cfg(feature = "defmt")]
    #[inline]
    pub fn log_router_lagged(count: u64) {
        defmt::error!("Router subscriber lagged by {} messages", count);
    }

    #[cfg(not(feature = "defmt"))]
    #[inline]
    pub fn log_router_lagged(_count: u64) {}

    #[cfg(feature = "defmt")]
    #[macro_export]
    macro_rules! __impl_defmt_format {
        ($enum_name:ident { $($variant:ident = $name:literal),* $(,)? }) => {
            impl $crate::__private::defmt::Format for $enum_name {
                fn format(&self, f: $crate::__private::defmt::Formatter) {
                    match self {
                        $($enum_name::$variant => $crate::__private::defmt::write!(f, $name)),*
                    }
                }
            }
        };
    }

    #[cfg(not(feature = "defmt"))]
    #[macro_export]
    macro_rules! __impl_defmt_format {
        ($enum_name:ident { $($variant:ident = $name:literal),* $(,)? }) => {};
    }

    #[doc(hidden)]
    pub use crate::__impl_defmt_format as impl_defmt_format;
}
