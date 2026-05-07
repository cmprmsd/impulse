//! This module should houses all horizontal/cross-cutting AI functionality throughout
//! Warp (including Agent Mode).
//!
//! The side panel Warp AI implementation lives in `super::ai_assistant`.
pub(crate) mod agent;
pub(crate) mod agent_events;
pub mod artifacts;
pub(crate) mod attachment_utils;
pub(crate) mod block_context;
pub(crate) mod blocklist;
pub mod control_code_parser;
pub(crate) mod conversation_status_ui;
pub(crate) mod conversation_utils;
pub(crate) mod document;
pub(crate) mod get_relevant_files;
pub(crate) mod llms;
pub(crate) mod predict;
pub(crate) mod skills;
pub(crate) mod voice;
use warpui::AppContext;
pub mod execution_profiles;
pub mod facts;
pub(crate) mod generate_block_title;
pub(crate) mod generate_code_review_content;
pub(crate) mod loading;
pub mod mcp;
pub mod outline;

pub(crate) use ai::paths;

pub fn init(app: &mut AppContext) {
    blocklist::keyboard_navigable_buttons::init(app);
    blocklist::block::number_shortcut_buttons::init(app);
    blocklist::toggleable_items::init(app);
    blocklist::suggested_agent_mode_workflow_modal::init(app);
    blocklist::suggested_rule_modal::init(app);
}
