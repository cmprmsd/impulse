//! Placeholder types for cloud-coupled code that hasn't been
//! rewritten on the local Phase 2 backend yet.
//!
//! After Phase 0 deleted the cloud crates (firebase, graphql,
//! warp_server_client, managed_secrets, …), the app crate was left
//! with hundreds of references to types that lived in those crates:
//! `SyncId`, `ServerId`, `CloudObjectTypeAndId`, `BlocklistAIHistoryModel`,
//! `UserWorkspaces`, etc. These types are not functional in the local-
//! only fork; the Phase 2 model_client/conversation backend in
//! `crates/ai/` is what replaces them.
//!
//! This module defines the *minimum* placeholder shapes (mostly
//! unit structs with `Default` + `Clone` + `Debug`) so the surrounding
//! code parses and type-checks. Calls into these stubs are dead at
//! runtime — when a cloud-coupled UI surface is exercised it will
//! hit a `todo!()` or no-op. Real implementations will replace these
//! one-by-one as Phase 2 wiring lands.

#![allow(dead_code, non_camel_case_types, unused_variables)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Identifier types (formerly in warp_server_client::ids)
// ---------------------------------------------------------------------------

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct ServerId(pub Uuid);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct ClientId(pub Uuid);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub enum SyncId {
    ClientId(ClientId),
    ServerId(ServerId),
}

impl Default for SyncId {
    fn default() -> Self {
        SyncId::ClientId(ClientId::default())
    }
}

impl SyncId {
    pub fn uid(&self) -> ObjectUid {
        ObjectUid::default()
    }
    pub fn sqlite_uid_hash(&self, _: ObjectIdType) -> HashedSqliteId {
        HashedSqliteId::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct ObjectUid(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct HashedSqliteId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct UserUid(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct FolderId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct WorkspaceUid(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct AmbientAgentTaskId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct AgentConversationEntryId(pub Uuid);

// ---------------------------------------------------------------------------
// Cloud-object enums (formerly in warp_server_client::cloud_object)
// ---------------------------------------------------------------------------

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub enum ObjectIdType {
    #[default]
    Notebook,
    Workflow,
    Folder,
    GenericStringObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectType {
    Notebook,
    Workflow,
    Folder,
    GenericStringObject(GenericStringObjectFormat),
}

impl Default for ObjectType {
    fn default() -> Self {
        ObjectType::Notebook
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum GenericStringObjectFormat {
    #[default]
    JsonEnvVarCollection,
    JsonAIFact,
    JsonMCPServer,
    JsonAIExecutionProfile,
    JsonTemplatableMCPServer,
    JsonCloudEnvironment,
    JsonScheduledAmbientAgent,
    JsonPreference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct JsonObjectType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CloudObjectTypeAndId {
    Notebook(SyncId),
    Workflow(SyncId),
    Folder(SyncId),
    GenericStringObject {
        object_type: GenericStringObjectFormat,
        id: SyncId,
    },
}

impl CloudObjectTypeAndId {
    pub fn from_id_and_type(id: SyncId, object_type: ObjectType) -> Self {
        match object_type {
            ObjectType::Notebook => Self::Notebook(id),
            ObjectType::Workflow => Self::Workflow(id),
            ObjectType::Folder => Self::Folder(id),
            ObjectType::GenericStringObject(format) => Self::GenericStringObject {
                object_type: format,
                id,
            },
        }
    }
    pub fn uid(self) -> ObjectUid {
        ObjectUid::default()
    }
    pub fn sync_id(self) -> SyncId {
        match self {
            Self::Notebook(id)
            | Self::Workflow(id)
            | Self::Folder(id)
            | Self::GenericStringObject { id, .. } => id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DriveObjectType {
    #[default]
    Workflow,
    AgentModeWorkflow,
    AIFact,
    AIFactCollection,
    Notebook { is_ai_document: bool },
    Folder,
    EnvVarCollection,
    MCPServer,
    MCPServerCollection,
}

#[derive(Debug, Clone, Default)]
pub struct Owner;

#[derive(Debug, Clone, Default)]
pub struct Space;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Revision(pub i64);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCreationInfo;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CloudObjectMetadata;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RevisionAndLastEditor;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerTimestamp;

// ---------------------------------------------------------------------------
// Telemetry (formerly in app::server::telemetry)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TelemetryEvent;

impl TelemetryEvent {
    pub fn name(&self) -> String {
        String::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteSource {
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnonymousUserSignupEntrypoint {
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharingDialogSource {
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseTarget {
    Default,
}

#[derive(Debug, Clone)]
pub enum AgentModeEntrypoint {
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentModeEntrypointSelectionType {
    Default,
}

// ---------------------------------------------------------------------------
// Auth (formerly in app::auth, deleted)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct AuthManager;

#[derive(Debug, Clone, Default)]
pub struct AuthState {
    pub is_authenticated: bool,
}

#[derive(Debug, Clone, Default)]
pub struct AuthStateProvider;

impl AuthStateProvider {
    pub fn current(&self) -> AuthState {
        AuthState::default()
    }
}

// ---------------------------------------------------------------------------
// ServerApi / cloud sync (formerly in app::server, deleted)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct ServerApi;

#[derive(Debug, Clone, Default)]
pub struct ServerApiProvider;

#[derive(Debug, Clone, Default)]
pub struct CloudModel;

#[derive(Debug, Clone)]
pub enum CloudModelEvent {
    Updated,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateManager;

#[derive(Debug, Clone)]
pub enum UpdateManagerEvent {
    Updated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectOperation {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationSuccessType {
    Default,
}

// ---------------------------------------------------------------------------
// Workspaces / teams (formerly in app::workspaces, deleted)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct UserWorkspaces;

#[derive(Debug, Clone, Default)]
pub struct WorkspaceMetadata;

#[derive(Debug, Clone, Default)]
pub struct UserProfileWithUID;

#[derive(Debug, Clone, Default)]
pub struct ServerExperiment;

#[derive(Debug, Clone, Default)]
pub struct TeamUpdateManager;

#[derive(Debug, Clone, Default)]
pub struct UserProfiles;

#[derive(Debug, Clone, Default)]
pub struct TeamTesterStatus;

// ---------------------------------------------------------------------------
// Remote-server bits (formerly in crates/remote_server, deleted)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct RemoteServerManager;

#[derive(Debug, Clone)]
pub enum RemoteServerManagerEvent {
    Connected,
    Disconnected,
}

#[derive(Debug, Clone, Default)]
pub struct RemoteServerInitPhase;

// ---------------------------------------------------------------------------
// Drive objects (formerly cloud_object module)
// ---------------------------------------------------------------------------

pub trait CloudObject: Send + Sync + std::fmt::Debug {}

// ---------------------------------------------------------------------------
// Blocklist (deleted; was the cloud-agent history model)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct BlocklistAIHistoryModel;

#[derive(Debug, Clone, Default)]
pub struct ObjectAction;

// ---------------------------------------------------------------------------
// Misc cloud bits referenced from app
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AISettings {
    #[default]
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarpifySettings {
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsAction {
    #[default]
    Default,
}

#[derive(Debug, Clone, Default)]
pub struct ToggleSettingActionPair;

#[derive(Debug, Clone, Default)]
pub struct CodebaseIndexManager;

#[derive(Debug, Clone, Default)]
pub struct OpenableFileType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSection {
    #[default]
    General,
}

#[derive(Debug, Clone, Default)]
pub struct UpstreamMessage;

#[derive(Debug, Clone, Default)]
pub struct DriveIndexEvent;

// Maps used in legacy persistence schema (to keep schema parsing).
pub type EnablementState = String;

// ---------------------------------------------------------------------------
// More deleted-cloud types found by the second cargo check sweep
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ContextChipKind {
    #[default]
    Default,
}

#[derive(Debug, Clone, Default)]
pub struct ChipValue;

#[derive(Debug, Clone, Default)]
pub struct LaunchConfig;

#[derive(Debug, Clone, Default)]
pub struct AIClient;

#[derive(Debug, Clone)]
pub enum AgentRunEvent {
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AgentModeCommandExecutionPredicate {
    #[default]
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct GenericStringObjectId(pub Uuid);

#[derive(Debug, Clone, Default)]
pub struct ApiKeyUid(pub String);

#[derive(Debug, Clone, Default)]
pub struct CellType;

#[derive(Debug, Clone, Default)]
pub struct ChannelState;

impl ChannelState {
    pub fn server_root_url() -> String {
        String::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct PaneViewLocator;

// ---------------------------------------------------------------------------
// Third sweep: agent SDK types referenced after the use-strip
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct AIConversationId(pub Uuid);

#[derive(Debug, Clone, Default)]
pub struct AIConversation;

#[derive(Debug, Clone, Default)]
pub struct AgentViewController;

#[derive(Debug, Clone, Default)]
pub struct AgentViewState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConversationStatus {
    #[default]
    Active,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentToolbarItemKind {
    #[default]
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

#[derive(Debug, Clone, Default)]
pub struct ServerConversationToken;

#[derive(Debug, Clone, Default)]
pub struct ConversationNavigationData;

#[derive(Debug, Clone, Default)]
pub struct QueueItem;

#[derive(Debug, Clone, Default)]
pub struct LspRepoStatus;

#[derive(Debug, Clone, Default)]
pub struct ModelAsRef;

#[derive(Debug, Clone, Default)]
pub struct ResponseEvent;

#[derive(Debug, Clone, Default)]
pub struct SettingsWidget;

#[derive(Debug, Clone, Default)]
pub struct AIAgentHarness;

#[derive(Debug, Clone, Default)]
pub struct CloudConversationData;

// ---------------------------------------------------------------------------
// Fourth sweep: more deleted cloud types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct CloudObjectLocation;

#[derive(Debug, Clone, Default)]
pub struct GenericCloudObject;

#[derive(Debug, Clone, Default)]
pub struct ServerCloudObject;

#[derive(Debug, Clone, Default)]
pub struct GenericStringObjectUniqueKey;

#[derive(Debug, Clone, Default)]
pub struct ShareableLinkError;

#[derive(Debug, Clone, Default)]
pub struct AmbientAgentViewModel;

#[derive(Debug, Clone, Default)]
pub struct StringModel;

#[derive(Debug, Clone, Default)]
pub struct DisplaySetting;

#[derive(Debug, Clone, Default)]
pub struct LoginGatedFeature;

#[derive(Debug, Clone, Default)]
pub struct JsonSerializer;

#[derive(Debug, Clone, Default)]
pub struct SecretHandle;

#[derive(Debug, Clone, Default)]
pub struct AgentModeCitation;

#[derive(Debug, Clone, Default)]
pub struct CreateCloudObjectResult;

#[derive(Debug, Clone, Default)]
pub struct CreateObjectRequest;

#[derive(Debug, Clone, Default)]
pub struct CloudObjectSyncStatus;

#[derive(Debug, Clone, Default)]
pub struct CloudModelType;

#[derive(Debug, Clone, Default)]
pub struct CloudObjectEventEntrypoint;

#[derive(Debug, Clone, Default)]
pub struct StringWithRequestId;

#[derive(Debug, Clone, Default)]
pub struct OrchestrationCardConfig;

#[derive(Debug, Clone, Default)]
pub struct AmbientAgentTask;

#[derive(Debug, Clone, Default)]
pub struct CloudAIFactModel;

#[derive(Debug, Clone, Default)]
pub struct CloudAIFact;

#[derive(Debug, Clone, Default)]
pub struct AIFact;

#[derive(Debug, Clone, Default)]
pub struct AIMemory;

#[derive(Debug, Clone, Default)]
pub struct DriveObjectPayload;

#[derive(Debug, Clone, Default)]
pub struct CloudEnvVarCollection;

#[derive(Debug, Clone, Default)]
pub struct WarpDriveItemId;

#[derive(Debug, Clone, Default)]
pub struct WarpDriveItem;

#[derive(Debug, Clone, Default)]
pub struct DriveIndexVariant;

#[derive(Debug, Clone, Default)]
pub struct ShareableObject;

#[derive(Debug, Clone, Default)]
pub struct OpenWarpDriveObjectArgs;

#[derive(Debug, Clone, Default)]
pub struct OpenWarpDriveObjectSettings;

#[derive(Debug, Clone, Default)]
pub struct ObjectClient;
