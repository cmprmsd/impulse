//! Stub macros for the cloud-coupled `warp_core::safe_*` and
//! `warp_core::send_telemetry_from_*` macros.
//!
//! The originals were two-arm `safe: (...) full: (...)` macros that
//! discriminated on the running channel. The stubs collapse to the
//! plain `log::*` equivalents and discard the telemetry args entirely
//! — telemetry has no destination after the cloud detach.
//!
//! Crate-wide visibility is achieved with `#[macro_export]` + a
//! `pub use` re-export at the crate root in `lib.rs`.

#[macro_export]
macro_rules! safe_debug {
    (safe: ($($safe_arg:tt)+), full: ($($full_arg:tt)+)) => {
        log::debug!($($full_arg)+)
    };
    ($($arg:tt)+) => { log::debug!($($arg)+) };
}

#[macro_export]
macro_rules! safe_info {
    (safe: ($($safe_arg:tt)+), full: ($($full_arg:tt)+)) => {
        log::info!($($full_arg)+)
    };
    ($($arg:tt)+) => { log::info!($($arg)+) };
}

#[macro_export]
macro_rules! safe_warn {
    (safe: ($($safe_arg:tt)+), full: ($($full_arg:tt)+)) => {
        log::warn!($($full_arg)+)
    };
    ($($arg:tt)+) => { log::warn!($($arg)+) };
}

#[macro_export]
macro_rules! safe_error {
    (safe: ($($safe_arg:tt)+), full: ($($full_arg:tt)+)) => {
        log::error!($($full_arg)+)
    };
    ($($arg:tt)+) => { log::error!($($arg)+) };
}

#[macro_export]
macro_rules! report_error {
    ($($arg:tt)+) => { log::error!($($arg)+) };
}

#[macro_export]
macro_rules! report_if_error {
    ($result:expr) => {
        if let Err(e) = $result {
            log::error!("{e}");
        }
    };
    ($result:expr, $($arg:tt)+) => {
        if let Err(e) = $result {
            log::error!($($arg)+);
        }
    };
}

#[macro_export]
macro_rules! send_telemetry_from_ctx {
    ($($arg:tt)*) => { () };
}

#[macro_export]
macro_rules! send_telemetry_from_app_ctx {
    ($($arg:tt)*) => { () };
}

#[macro_export]
macro_rules! send_telemetry_on_executor {
    ($($arg:tt)*) => { () };
}

#[macro_export]
macro_rules! send_telemetry_sync_from_app_ctx {
    ($($arg:tt)*) => { () };
}

/// `id!("FooView")` re-export from `warpui::keymap::macros::id`.
/// Made crate-wide via `#[macro_use]` instead of per-file
/// `use warpui::keymap::macros::id;` in scope.
#[macro_export]
macro_rules! id {
    ($val:literal) => {
        warpui::keymap::ContextPredicate::Identifier($val)
    };
    ($val:expr) => {
        warpui::keymap::ContextPredicate::Identifier($val)
    };
}

/// `eq!("key", "value")` re-export.
#[macro_export]
macro_rules! eq {
    ($a:literal, $b:literal) => {
        warpui::keymap::ContextPredicate::Equal($a, $b)
    };
}

/// `ne!("key", "value")` re-export.
#[macro_export]
macro_rules! ne {
    ($a:literal, $b:literal) => {
        warpui::keymap::ContextPredicate::NotEqual($a, $b)
    };
}
