//! Stub for the deleted `api` (multi-agent client_action / response_event) crate.

pub mod client_action {}
pub mod response_event {
    pub mod stream_finished {}
}

#[derive(Debug, Clone, Default)]
pub struct RequestParams;

pub mod review_comment {
    #[derive(Debug, Clone, Default)]
    pub struct ReviewComment;
}

pub mod agent_event {
    #[derive(Debug, Clone, Default)]
    pub struct AgentEvent;

    #[derive(Debug, Clone, Default)]
    pub struct StreamFinished;

    #[derive(Debug, Clone, Default)]
    pub struct AssistantMessage;

    #[derive(Debug, Clone)]
    pub enum Type {
        StreamFinished(StreamFinished),
        AssistantMessage(AssistantMessage),
    }

    pub mod lifecycle_event {
        #[derive(Debug, Clone, Default)]
        pub struct Started;
        #[derive(Debug, Clone, Default)]
        pub struct Completed;
        #[derive(Debug, Clone, Default)]
        pub struct Failed;
        #[derive(Debug, Clone, Default)]
        pub struct Cancelled;
        #[derive(Debug, Clone)]
        pub enum Type {
            Started(Started),
            Completed(Completed),
            Failed(Failed),
            Cancelled(Cancelled),
        }
    }
}
pub mod message {
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
