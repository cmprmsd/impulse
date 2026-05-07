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

// safe_* macros: warp_core provides them via #[macro_export]; we don't redefine
// here to avoid E0659 ambiguity at every call site that has
// `use warp_core::safe_*;` at file level.

// send_telemetry_* macros: stub them at the crate root via #[macro_export].
// Files that previously used these via `use warp_core::send_telemetry_*;`
// must drop those imports to avoid E0659 ambiguity (handled by a sweep
// in this commit).
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

#[macro_export]
macro_rules! send_telemetry_sync_from_ctx {
    ($($arg:tt)*) => { () };
}

// id!, eq!, ne! macros are NOT redefined here. The warpui crate
// (via warpui_core::keymap::macros) re-exports them, and any file
// that needs them can `use warpui::keymap::macros::*;` inside the
// scope where they're called. Defining stubs at the crate root via
// #[macro_export] caused E0659 ambiguity errors at every id!() call
// site.
