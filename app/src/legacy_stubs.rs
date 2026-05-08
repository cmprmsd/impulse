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

#[derive(Debug, Clone)]
pub enum UserWorkspacesEvent {
    Updated,
}

#[derive(Debug, Clone, Default)]
pub struct Team;

#[derive(Debug, Clone, Default)]
pub struct Workspace;

#[derive(Debug, Clone, Default)]
pub enum CustomerType {
    #[default]
    Personal,
    Team,
}

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

/// Stub trait so call sites that expect `dyn AIClient` parse.
/// Methods are stubbed to return errors at runtime.
pub trait AIClient: Send + Sync {
    fn read_agent_message<'a>(
        &'a self,
        _message_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<ReadAgentMessageResponse>> + Send + 'a>> {
        Box::pin(async {
            Err(anyhow::anyhow!("AIClient is a stub; not implemented in OSS fork"))
        })
    }
    fn mark_message_delivered<'a>(
        &'a self,
        _message_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(async {
            Err(anyhow::anyhow!("AIClient is a stub; not implemented in OSS fork"))
        })
    }
}

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
pub struct GenericCloudObject<I = (), M = ()>(std::marker::PhantomData<(I, M)>);

#[derive(Debug, Clone, Default)]
pub struct ServerCloudObject<I = (), M = ()>(std::marker::PhantomData<(I, M)>);

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

// ---------------------------------------------------------------------------
// Fifth sweep: more cloud-side helpers seen in remaining errors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct ServerAIConversationMetadata;

#[derive(Debug, Clone, Default)]
pub struct ReviewComment;

#[derive(Debug, Clone, Default)]
pub struct RenderableOptionConfig;

#[derive(Debug, Clone, Default)]
pub struct JsonModel;

#[derive(Debug, Clone, Default)]
pub struct GenericStringModel<T = (), S = ()>(std::marker::PhantomData<(T, S)>);

#[derive(Debug, Clone, Default)]
pub struct ServerTime;

#[derive(Debug, Clone, Default)]
pub struct MessageProvider;

#[derive(Debug, Clone, Default)]
pub struct SizeInfo;

#[derive(Debug, Clone, Default)]
pub struct PaneTemplateType;

#[derive(Debug, Clone, Default)]
pub struct NetworkLogView;

#[derive(Debug, Clone, Default)]
pub struct SubmittableTextInput;

#[derive(Debug, Clone, Default)]
pub struct TipsCompleted;

#[derive(Debug, Clone, Default)]
pub struct DetectedLinksState;

#[derive(Debug, Clone, Default)]
pub struct CustomSecretRegexUpdater;

impl CustomSecretRegexUpdater {
    pub fn redact_secrets<S: AsRef<str>>(s: S) -> String {
        s.as_ref().to_string()
    }
}

pub fn redact_secrets<S: AsRef<str>>(s: S) -> String {
    s.as_ref().to_string()
}

#[derive(Debug, Clone, Default)]
pub struct AmbientAgentTaskInput;

#[derive(Debug, Clone, Default)]
pub struct AmbientAgentTaskMetadata;

#[derive(Debug, Clone, Default)]
pub struct UserContextMetadata;

#[derive(Debug, Clone, Default)]
pub struct ScheduledAmbientAgent;

#[derive(Debug, Clone, Default)]
pub struct AgentRun;

#[derive(Debug, Clone, Default)]
pub struct StartAgentExecutionMode;

#[derive(Debug, Clone, Default)]
pub struct AIAgentExecutionProfileFields;

#[derive(Debug, Clone, Default)]
pub struct AgentEnvironment;

#[derive(Debug, Clone, Default)]
pub struct ScheduledAgentTaskRunHistory;

#[derive(Debug, Clone, Default)]
pub struct PassiveSuggestionTriggerType;

#[derive(Debug, Clone, Default)]
pub struct EntrypointType;

#[derive(Debug, Clone, Default)]
pub struct OutputModelInfo;

#[derive(Debug, Clone, Default)]
pub struct ChannelStateEvent;

#[derive(Debug, Clone, Default)]
pub struct ServerEnvironment;

#[derive(Debug, Clone, Default)]
pub struct AmbientAgentDispatchSource;

#[derive(Debug, Clone, Default)]
pub struct DispatchAmbientAgentRequest;

// ---------------------------------------------------------------------------
// Sixth sweep: more types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct ExternalSecret;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InitiatedBy {
    #[default]
    User,
    Agent,
}

#[derive(Debug, Clone, Default)]
pub struct ContainingObject;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CLIAgentType {
    #[default]
    Default,
}

#[derive(Debug, Clone, Default)]
pub struct PaintContext;

#[derive(Debug, Clone)]
pub enum MenuEvent {
    Selected,
    Closed,
}

#[derive(Debug, Clone, Default)]
pub struct EphemeralMessageModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContentEditability {
    #[default]
    Editable,
    ReadOnly,
}

#[derive(Debug, Clone, Default)]
pub struct TaskStatusUpdate;

#[derive(Debug, Clone, Default)]
pub struct StoredCredentials;

#[derive(Debug, Clone, Default)]
pub struct SpawnedFutureHandle;

#[derive(Debug, Clone, Default)]
pub struct VisibleRow;

#[derive(Debug, Clone, Default)]
pub struct SizeConstraint;

// ---------------------------------------------------------------------------
// Seventh sweep
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct SharingDialog;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SharingAccessLevel {
    #[default]
    Read,
    Write,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedTemplatableMCPServerResult;

#[derive(Debug, Clone, Default)]
pub struct ImportQueueArgs;

#[derive(Debug, Clone, Default)]
pub struct CloudStringObject;

#[derive(Debug, Clone, Default)]
pub struct CloudObjectTelemetryMetadata;

#[derive(Debug, Clone, Default)]
pub struct BlockClient;

#[derive(Debug, Clone, Default)]
pub struct AuthError;

#[derive(Debug, Clone, Default)]
pub struct AuthClient;

#[derive(Debug, Clone, Default)]
pub struct AttachmentInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InteractionSource {
    #[default]
    Default,
}

pub const GENERIC_STRING_OBJECT_PREFIX: &str = "obj_";

#[derive(Debug, Clone, Default)]
pub struct CodeReviewModel;

#[derive(Debug, Clone, Default)]
pub struct GlobalCodeReviewModel;

#[derive(Debug, Clone, Default)]
pub struct AgentSdkProvider;

#[derive(Debug, Clone, Default)]
pub struct AgentManagement;

#[derive(Debug, Clone, Default)]
pub struct AmbientAgentLifecycle;

#[derive(Debug, Clone, Default)]
pub struct CodebaseIndexingState;

#[derive(Debug, Clone, Default)]
pub struct AgentMetadata;

#[derive(Debug, Clone, Default)]
pub struct AgentMessage;

#[derive(Debug, Clone, Default)]
pub struct AgentTaskState;

#[derive(Debug, Clone, Default)]
pub struct ConversationOptions;

#[derive(Debug, Clone, Default)]
pub struct AIInputBlock;

#[derive(Debug, Clone, Default)]
pub struct AICommandBlock;

#[derive(Debug, Clone, Default)]
pub struct AIOutput;

#[derive(Debug, Clone, Default)]
pub struct AIOutputId;

// ---------------------------------------------------------------------------
// Eighth sweep
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderState {
    #[default]
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InlineMenuType {
    #[default]
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GridType {
    #[default]
    Default,
}

#[derive(Debug, Clone, Default)]
pub struct FuzzyMatchResult;

#[derive(Debug, Clone, Default)]
pub struct FindOptions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArgumentType {
    #[default]
    String,
}

#[derive(Debug, Clone, Default)]
pub struct AgentInputFooter;

#[derive(Debug, Clone, Default)]
pub struct AgentConversationEntry;

#[derive(Debug, Clone, Default)]
pub struct AIApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkspaceDecorationVisibility {
    #[default]
    Default,
}

#[derive(Debug, Clone, Default)]
pub struct UploadIntent;

#[derive(Debug, Clone, Default)]
pub struct UploadId;

#[derive(Debug, Clone, Default)]
pub struct UnlinkRunArgs;

#[derive(Debug, Clone, Default)]
pub struct ToolCall;

#[derive(Debug, Clone, Default)]
pub struct TimedSession;

#[derive(Debug, Clone, Default)]
pub struct TerminationGracePeriod;

#[derive(Debug, Clone, Default)]
pub struct StreamFinishedReason;

#[derive(Debug, Clone, Default)]
pub struct SkillReference;

#[derive(Debug, Clone, Default)]
pub struct ScheduleArgs;

#[derive(Debug, Clone, Default)]
pub struct ScheduleAndDispatch;

#[derive(Debug, Clone, Default)]
pub struct RunWithStartingSnapshot;

#[derive(Debug, Clone, Default)]
pub struct RunActionArgs;

#[derive(Debug, Clone, Default)]
pub struct ResumeConversationArgs;

#[derive(Debug, Clone, Default)]
pub struct RegisterMacroArgs;

#[derive(Debug, Clone, Default)]
pub struct PrTaskCounts;

#[derive(Debug, Clone, Default)]
pub struct PathRoot;

#[derive(Debug, Clone, Default)]
pub struct PassiveSuggestionTrigger;

#[derive(Debug, Clone, Default)]
pub struct MCPServerTelemetryError;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentSource {
    #[default]
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LaunchConfigUiLocation {
    #[default]
    Default,
}

#[derive(Debug, Clone, Default)]
pub struct ReadAgentMessageResponse {
    pub message_id: String,
    pub body: String,
    pub sender_run_id: String,
    pub subject: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ArtifactDownloadResponse;

impl ArtifactDownloadResponse {
    pub fn artifact_uid(&self) -> &str { "" }
    pub fn download_url(&self) -> String { String::new() }
}

#[derive(Debug, Clone, Default)]
pub struct CitationForTelemetry {
    pub object_type: ObjectType,
    pub uid: ObjectUid,
}

impl CitationForTelemetry {
    pub fn warp_drive_object(object_type: ObjectType, uid: ObjectUid) -> Self {
        Self { object_type, uid }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AIAgentExchangeId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServerOutputId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AIDocumentId(pub Uuid);
