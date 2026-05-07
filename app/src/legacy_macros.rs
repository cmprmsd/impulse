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

// send_telemetry_* macros are NOT redefined here — warp_core already
// exports them with `#[macro_export]`, so the crate-root path
// `crate::send_telemetry_from_ctx` works. Defining duplicates here
// caused E0659 ambiguity at every call site.

// id!, eq!, ne! macros are NOT redefined here. The warpui crate
// (via warpui_core::keymap::macros) re-exports them, and any file
// that needs them can `use warpui::keymap::macros::*;` inside the
// scope where they're called. Defining stubs at the crate root via
// #[macro_export] caused E0659 ambiguity errors at every id!() call
// site.
