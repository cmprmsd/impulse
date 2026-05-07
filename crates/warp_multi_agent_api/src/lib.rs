//! Stub crate replacing the deleted `warp_multi_agent_api` (proto-generated).
//! Provides the minimum types still referenced by call sites in `app/src/`
//! and `crates/persistence/`.

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Task {
    pub dependencies: Vec<String>,
}

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
            pub total_tokens: i32,
            pub token_usage_by_category: HashMap<String, i32>,
        }

        #[derive(Debug, Clone, Default)]
        pub struct ToolCallStats {
            pub count: i32,
        }

        #[derive(Debug, Clone, Default)]
        pub struct RunCommandStats {
            pub count: i32,
            pub command_executed: i32,
        }

        #[derive(Debug, Clone, Default)]
        pub struct ApplyFileDiffStats {
            pub count: i32,
            pub lines_added: i32,
            pub lines_removed: i32,
            pub files_changed: i32,
        }

        #[derive(Debug, Clone, Default)]
        pub struct ToolUsageMetadata {
            pub run_command_stats: Option<RunCommandStats>,
            pub read_files_stats: Option<ToolCallStats>,
            pub search_codebase_stats: Option<ToolCallStats>,
            pub grep_stats: Option<ToolCallStats>,
            pub file_glob_stats: Option<ToolCallStats>,
            pub apply_file_diff_stats: Option<ApplyFileDiffStats>,
            pub write_to_long_running_shell_command_stats: Option<ToolCallStats>,
            pub read_shell_command_output_stats: Option<ToolCallStats>,
            pub read_mcp_resource_stats: Option<ToolCallStats>,
            pub call_mcp_tool_stats: Option<ToolCallStats>,
            pub suggest_plan_stats: Option<ToolCallStats>,
            pub suggest_create_plan_stats: Option<ToolCallStats>,
            pub use_computer_stats: Option<ToolCallStats>,
        }
    }
}


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
