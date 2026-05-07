//! Stub crate replacing the deleted `warp_multi_agent_api` (proto-generated).
//! Provides the minimum types still referenced by call sites in `app/src/`
//! and `crates/persistence/`.

#[derive(Debug, Clone, Default)]
pub struct Task;

pub mod response_event {
    #[derive(Debug, Clone)]
    pub enum Type {
        Init(InitEvent),
        Finished(StreamFinished),
        ClientActions(ClientActions),
    }

    #[derive(Debug, Clone, Default)]
    pub struct InitEvent;

    #[derive(Debug, Clone, Default)]
    pub struct StreamFinished {
        pub reason: Option<stream_finished::Reason>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct ClientActions;

    pub mod stream_finished {
        use std::collections::HashMap;

        #[derive(Debug, Clone)]
        pub enum Reason {
            Done(()),
            Other(()),
            ContextWindowExceeded(()),
            QuotaLimit(()),
            LlmUnavailable(()),
        }

        #[derive(Debug, Clone, Default)]
        pub struct ModelTokenUsage {
            pub model_id: String,
            pub total_tokens: u32,
            pub token_usage_by_category: HashMap<crate::TokenUsageCategory, u32>,
        }

        #[derive(Debug, Clone, Default)]
        pub struct ToolCallStats {
            pub tool_name: String,
            pub call_count: u32,
        }

        #[derive(Debug, Clone, Default)]
        pub struct RunCommandStats {
            pub command: String,
            pub run_count: u32,
        }

        #[derive(Debug, Clone, Default)]
        pub struct ApplyFileDiffStats {
            pub file_path: String,
            pub diff_count: u32,
        }

        #[derive(Debug, Clone, Default)]
        pub struct ToolUsageMetadata {
            pub tool_name: String,
            pub usage_count: u32,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct TokenUsageCategory(pub String);

pub mod base_ref {
    #[derive(Debug, Clone)]
    pub enum Ref {
        BranchName(String),
        HeadlessCommitSha(String),
        UncommittedChanges(()),
    }
}

#[derive(Debug, Clone, Default)]
pub struct BaseRef {
    pub r#ref: Option<base_ref::Ref>,
}

pub mod diff_set {
    #[derive(Debug, Clone, Default)]
    pub struct DiffHunk {
        pub file_path: String,
        pub line_range: Option<super::FileContentLineRange>,
        pub diff_content: String,
        pub lines_added: u32,
        pub lines_removed: u32,
    }
}

#[derive(Debug, Clone, Default)]
pub struct FileContentLineRange {
    pub start: u32,
    pub end: u32,
}
