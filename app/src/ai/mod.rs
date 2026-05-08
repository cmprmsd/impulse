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

// Stubs for cloud-only submodules removed during the cloud strip.
pub mod agent_sdk {
    pub mod artifact_upload {
        #[derive(Default)]
        pub struct FileArtifactUploadRequest;
        pub struct FileArtifactUploader;
    }
}
pub mod agent_management {}
pub mod conversation_details_panel {
    pub struct ConversationDetailsPanel;
}
pub mod persisted_workspace {
    use std::path::PathBuf;
    use lsp::supported_servers::LSPServerType;
    pub enum LspTask {
        Spawn { file_path: PathBuf },
        Install { file_path: PathBuf, repo_root: PathBuf, server_type: LSPServerType },
    }
}

// Cloud-only stubs widely referenced by call sites
pub use crate::legacy_stubs::{
    AIRequestUsageModel, AgentTip, BuyCreditsBannerDisplayState, RequestLimitInfo,
    RequestLimitRefreshDuration, RequestUsageInfo,
};

// Empty submodule placeholders for deleted cloud features
pub mod ambient_agents {
    pub use crate::legacy_stubs::AmbientAgentTaskId;

    #[derive(Debug, Clone, Default)]
    pub struct AgentConfigSnapshot;

    pub mod task {
        #[derive(Debug, Clone, Default)]
        pub struct AmbientAgentTask;
    }
}
pub mod active_agent_views_model {
    #[derive(Debug, Clone, Default)]
    pub struct ActiveAgentViewsModel;
}
pub mod agent_tips {
    pub use crate::legacy_stubs::AgentTip;
    pub struct AITipModel<T>(std::marker::PhantomData<T>);
    impl<T> AITipModel<T> {
        pub fn new_for_agent_tips<C>(_ctx: &mut C) -> Self {
            Self(std::marker::PhantomData)
        }
    }
}
pub mod ai_document_view {}
pub mod cloud_environments {}
pub mod harness_availability {}

pub fn init(app: &mut AppContext) {
    blocklist::keyboard_navigable_buttons::init(app);
    blocklist::block::number_shortcut_buttons::init(app);
    blocklist::toggleable_items::init(app);
    blocklist::suggested_agent_mode_workflow_modal::init(app);
    blocklist::suggested_rule_modal::init(app);
}
