//! Stub for the deleted `api` (multi-agent client_action / response_event) crate.

pub mod client_action {
    #[derive(Debug, Clone, Default)]
    pub struct CreateTask {
        pub task: Option<super::Task>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct AddMessagesToTask {
        pub task_id: String,
        pub messages: Vec<super::message::Message>,
    }

    #[derive(Debug, Clone)]
    pub enum Action {
        CreateTask(CreateTask),
        AddMessagesToTask(AddMessagesToTask),
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClientAction {
    pub action: Option<client_action::Action>,
}

#[derive(Debug, Clone, Default)]
pub struct Task {
    pub id: String,
    pub description: Option<String>,
    pub dependencies: Option<TaskDependencies>,
    pub messages: Vec<message::Message>,
    pub summary: Option<String>,
    pub server_data: Option<TaskServerData>,
}

#[derive(Debug, Clone, Default)]
pub struct TaskDependencies {
    pub parent_task_id: String,
}

#[derive(Debug, Clone, Default)]
pub struct TaskServerData;

pub mod response_event {
    use super::ClientAction;

    #[derive(Debug, Clone, Default)]
    pub struct StreamInit {
        pub request_id: String,
        pub conversation_id: String,
        pub run_id: String,
    }

    #[derive(Debug, Clone, Default)]
    pub struct ClientActions {
        pub actions: Vec<ClientAction>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct StreamFinished {
        pub reason: Option<stream_finished::Reason>,
        pub conversation_usage_metadata: Option<stream_finished::ConversationUsageMetadata>,
        pub token_usage: Vec<stream_finished::ModelTokenUsage>,
        pub should_refresh_model_config: bool,
        pub request_cost: Option<()>,
    }

    #[derive(Debug, Clone)]
    pub enum Type {
        Init(StreamInit),
        ClientActions(ClientActions),
        Finished(StreamFinished),
    }

    pub mod stream_finished {
        #[derive(Debug, Clone)]
        pub enum Reason {
            Done(Done),
            Other(()),
            ContextWindowExceeded(()),
            QuotaLimit(()),
            LlmUnavailable(()),
        }

        #[derive(Debug, Clone, Default)]
        pub struct Done {}

        #[derive(Debug, Clone, Default)]
        pub struct ConversationUsageMetadata {
            pub context_window_usage: Option<u32>,
            pub credits_spent: Option<f32>,
            pub summarized: bool,
            pub token_usage: Vec<ModelTokenUsage>,
            pub tool_usage_metadata: Option<ToolUsageMetadata>,
            pub warp_token_usage: Vec<ModelTokenUsage>,
            pub byok_token_usage: Vec<ModelTokenUsage>,
        }

        #[derive(Debug, Clone, Default)]
        pub struct ModelTokenUsage {
            pub model_id: String,
            pub total_tokens: u32,
        }

        #[derive(Debug, Clone, Default)]
        pub struct ToolUsageMetadata;
    }
}

pub mod message {
    #[derive(Debug, Clone, Default)]
    pub struct Message {
        pub request_id: String,
    }

    pub mod tool_call {
        #[derive(Debug, Clone, Default)]
        pub struct ToolCall;

        #[derive(Debug, Clone)]
        pub enum Tool {
            Default,
        }
    }

    pub mod artifact_event {
        #[derive(Debug, Clone, Default)]
        pub struct PullRequestArtifact {
            pub url: String,
            pub branch: String,
        }

        #[derive(Debug, Clone, Default)]
        pub struct ScreenshotArtifact {
            pub artifact_uid: String,
            pub mime_type: String,
            pub description: String,
        }

        #[derive(Debug, Clone, Default)]
        pub struct FileArtifact {
            pub artifact_uid: String,
            pub filepath: String,
            pub mime_type: String,
            pub description: String,
            pub size_bytes: u64,
        }

        #[derive(Debug, Clone, Default)]
        pub struct PlanArtifact {
            pub document_id: String,
            pub notebook_uid: String,
            pub title: String,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RequestParams;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEventType {
    Unspecified,
    Started,
    InProgress,
    Succeeded,
    Failed,
    Errored,
    Cancelled,
    Blocked,
    Restarted,
    Idle,
}

pub mod review_comment {
    #[derive(Debug, Clone, Default)]
    pub struct ReviewComment;
    #[derive(Debug, Clone)]
    pub enum CommentTarget {
        Default,
    }
}

pub mod agent_event {
    #[derive(Debug, Clone, Default)]
    pub struct AgentEvent;

    #[derive(Debug, Clone, Default)]
    pub struct StreamFinished;

    #[derive(Debug, Clone, Default)]
    pub struct AssistantMessage;

    #[derive(Debug, Clone, Default)]
    pub struct LifecycleEvent {
        pub detail: Option<lifecycle_event::Detail>,
    }

    #[derive(Debug, Clone)]
    pub enum Type {
        StreamFinished(StreamFinished),
        AssistantMessage(AssistantMessage),
        LifecycleEvent(LifecycleEvent),
    }

    #[derive(Debug, Clone)]
    pub enum Event {
        StreamFinished(StreamFinished),
        AssistantMessage(AssistantMessage),
        LifecycleEvent(LifecycleEvent),
    }

    pub mod lifecycle_event {
        #[derive(Debug, Clone, Default)]
        pub struct Started;
        #[derive(Debug, Clone, Default)]
        pub struct Completed;
        #[derive(Debug, Clone, Default)]
        pub struct Failed {
            pub message: String,
        }
        #[derive(Debug, Clone, Default)]
        pub struct Cancelled;
        #[derive(Debug, Clone, Default)]
        pub struct Succeeded;
        #[derive(Debug, Clone, Default)]
        pub struct InProgress;
        #[derive(Debug, Clone, Default)]
        pub struct Blocked;
        #[derive(Debug, Clone, Default)]
        pub struct Errored;
        #[derive(Debug, Clone)]
        pub enum Type {
            Started(Started),
            Completed(Completed),
            Failed(Failed),
            Cancelled(Cancelled),
            Succeeded(Succeeded),
            InProgress(InProgress),
            Blocked(Blocked),
            Errored(Errored),
        }
        #[derive(Debug, Clone)]
        pub enum Detail {
            Started(()),
            Completed(()),
            Failed(Failed),
            Cancelled(()),
            Succeeded(()),
            InProgress(()),
            Blocked(()),
            Errored(()),
        }
    }
}
