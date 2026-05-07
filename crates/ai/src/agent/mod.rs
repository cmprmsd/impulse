pub mod action;
pub mod action_result;
pub mod file_locations;
pub mod orchestration_config;
// Stubs for cloud-detached builds; original cloud-only modules removed.
pub mod todos {}
pub mod task {}
pub mod redaction {}
pub mod api {}

pub use action::AIAgentCitation;
pub use file_locations::{group_file_contexts_for_display, FileLocations};
