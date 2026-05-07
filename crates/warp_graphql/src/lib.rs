//! Stub for the deleted `warp_graphql` crate. Provides minimal types
//! still referenced by Phase-0-not-yet-cleaned-up call sites.

pub mod ai {
    use super::Wrapped;

    #[derive(Debug, Clone)]
    pub enum AIConversationArtifact {
        PlanArtifact(PlanArtifact),
        PullRequestArtifact(PullRequestArtifact),
        ScreenshotArtifact(ScreenshotArtifact),
        FileArtifact(FileArtifact),
        Unknown,
    }

    #[derive(Debug, Clone, Default)]
    pub struct PlanArtifact {
        pub document_uid: Wrapped,
        pub notebook_uid: Option<Wrapped>,
        pub title: Option<String>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct PullRequestArtifact {
        pub url: String,
        pub branch: String,
    }

    #[derive(Debug, Clone, Default)]
    pub struct ScreenshotArtifact {
        pub artifact_uid: Wrapped,
        pub mime_type: String,
        pub description: Option<String>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct FileArtifact {
        pub artifact_uid: Wrapped,
        pub filepath: String,
        pub mime_type: String,
        pub description: Option<String>,
        pub size_bytes: Option<i32>,
    }
}

#[derive(Debug, Clone, Default)]
pub struct Wrapped(pub String);

impl Wrapped {
    pub fn into_inner(self) -> String { self.0 }
}

pub mod mcp_gallery_template {
    #[derive(Debug, Clone, Default)]
    pub struct Template;
}

pub mod team {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MembershipRole {
        Default,
    }
}

pub mod queries {}
pub mod mutations {}
pub mod client {}
pub mod object_permissions {}
