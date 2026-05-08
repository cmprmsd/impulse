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

pub mod object_permissions {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum OwnerType {
        User,
        Team,
    }
}

#[derive(Debug, Clone, Default)]
pub struct GraphQLDateTime(pub chrono::DateTime<chrono::Utc>);

impl GraphQLDateTime {
    pub fn utc(&self) -> chrono::DateTime<chrono::Utc> {
        self.0
    }
}

impl<O: Into<chrono::DateTime<chrono::Utc>>> From<O> for GraphQLDateTime {
    fn from(o: O) -> Self {
        GraphQLDateTime(o.into())
    }
}

#[derive(Debug, Clone, Default)]
pub struct UserFacingErrorPayload {
    pub message: String,
}

pub mod client {
    use super::UserFacingErrorPayload;
    pub fn get_user_facing_error_message(e: UserFacingErrorPayload) -> String {
        e.message
    }
}

pub mod queries {
    pub mod api_keys {
        use super::super::{GraphQLDateTime, Wrapped, object_permissions::OwnerType};

        #[derive(Debug, Clone)]
        pub struct ApiKeyProperties {
            pub uid: Wrapped,
            pub name: String,
            pub key_suffix: String,
            pub owner_type: OwnerType,
            pub created_at: GraphQLDateTime,
            pub last_used_at: Option<GraphQLDateTime>,
            pub expires_at: Option<GraphQLDateTime>,
        }

        impl Default for ApiKeyProperties {
            fn default() -> Self {
                Self {
                    uid: Wrapped::default(),
                    name: String::new(),
                    key_suffix: String::new(),
                    owner_type: OwnerType::User,
                    created_at: GraphQLDateTime::default(),
                    last_used_at: None,
                    expires_at: None,
                }
            }
        }
    }

    pub mod suggest_cloud_environment_image {
        use super::super::UserFacingErrorPayload;

        #[derive(Debug, Clone, Default)]
        pub struct SuggestCloudEnvironmentImageOutput {
            pub image: String,
            pub needs_custom_image: bool,
            pub reason: String,
        }

        #[derive(Debug, Clone, Default)]
        pub struct SuggestCloudEnvironmentImageAuthRequiredOutput {
            pub auth_url: String,
        }

        #[derive(Debug, Clone)]
        pub enum SuggestCloudEnvironmentImageResult {
            SuggestCloudEnvironmentImageOutput(SuggestCloudEnvironmentImageOutput),
            SuggestCloudEnvironmentImageAuthRequiredOutput(
                SuggestCloudEnvironmentImageAuthRequiredOutput,
            ),
            UserFacingError(UserFacingErrorPayload),
            Unknown,
        }
    }
}

pub mod mutations {
    pub mod generate_api_key {
        use super::super::UserFacingErrorPayload;
        use super::super::queries::api_keys::ApiKeyProperties;

        #[derive(Debug, Clone, Default)]
        pub struct GenerateApiKeyOutput {
            pub api_key: ApiKeyProperties,
            pub raw_api_key: String,
        }

        #[derive(Debug, Clone)]
        pub enum GenerateApiKeyResult {
            GenerateApiKeyOutput(GenerateApiKeyOutput),
            UserFacingError(UserFacingErrorPayload),
            Unknown,
        }
    }

    pub mod expire_api_key {
        use super::super::UserFacingErrorPayload;

        #[derive(Debug, Clone, Default)]
        pub struct ExpireApiKeyOutput {
            pub uid: String,
        }

        #[derive(Debug, Clone)]
        pub enum ExpireApiKeyResult {
            ExpireApiKeyOutput(ExpireApiKeyOutput),
            UserFacingError(UserFacingErrorPayload),
            Unknown,
        }
    }
}
