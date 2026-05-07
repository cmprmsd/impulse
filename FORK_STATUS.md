# Warp OSS fork — current status

## Progress this session

`cargo check -p warp` reduced from **8314** to a few dozen residual
type-resolution errors (>99% reduction) across ~140 commits to
`claude/audit-warp-terminal-lNAJT`.

## Phase 0 (cloud strip) — ~99% complete

- Removed 7 cloud crates (firebase, graphql, warp_server_client,
  managed_secrets, warp_graphql_schema, original `remote_server`,
  `warp_multi_agent_api`) and ~250 cloud-only files.
- Replaced four of those with **local stub crates** under `crates/`:
  - `warp_multi_agent_api/` — Task, response_event::stream_finished
    (ModelTokenUsage, ToolCallStats, RunCommandStats, ApplyFileDiffStats,
    ToolUsageMetadata field-level), AgentType, ToolType, message,
    diff_set, BaseRef, FileContentLineRange, Skill.
  - `api/` — message::artifact_event, response_event,
    agent_event::lifecycle_event::{Type,Detail}, RequestParams,
    review_comment::{ReviewComment,CommentTarget}.
  - `remote_server/` — proto::{ReadFileContextRequest, …,
    file_context_proto::Content}, client::{RemoteServerClient,
    ReadFileContextResponse, FileContextProto, FailedFile}.
  - `warp_graphql/` — ai::AIConversationArtifact and variants,
    Wrapped, mcp_gallery_template, team::MembershipRole, queries,
    mutations, client, object_permissions.
- All parse errors resolved.

## Phase 2 (model_client foundation) — 100% complete

- `crates/ai/model_client/` (`OpenAiCompatibleClient`, `ClaudeCliClient`)
- `crates/ai/conversation/` (JSONL store under `~/WarpData/ai_conversations/`)
- `crates/ai/agent_loop.rs` orchestration
- `provider_settings::AiProvider` enum + `build_client`
- 101 unit tests + runnable `agent_demo.rs`

## What was done programmatically (~140 commits)

1. Brace-balance fixes (~940 inserts/removals across 1500+ files,
   tokenizer-aware indent-stack matching).
2. Comment unresolved imports (~1071 lines, 77 files).
3. Stub modules under `app/src/lib.rs` for `crate::server`,
   `crate::auth`, `crate::cloud_object`, `crate::workspaces`,
   `crate::remote_server`, `crate::launch_configs` (re-exporting from
   `legacy_stubs.rs`).
4. Macro hygiene: drop legacy `safe_*` and `send_telemetry_*` stubs;
   re-export warp_core's via `pub use`. Add explicit imports to ~150
   files where macros were used without imports.
5. Trait-type alignment: redirect `legacy_stubs::InlineMenuType` and
   `legacy_stubs::SettingsSection` imports to canonical paths.
6. Generic-args: `<I, M>` parameters on stub `GenericCloudObject`
   and `GenericStringModel`.
7. From-impl removal in `artifacts/mod.rs` and `agent/mod.rs` (their
   input crates were deleted).
8. Stub Cargo crates for the deleted protobuf crates (above).

## Phase 1 (local-folder Drive backend) — not started

`LocalFsObjectClient` impl of `ObjectClient` trait, backed by
`~/WarpData/`. Filesystem watcher to replace
`get_warp_drive_updates` subscription.

## Phase 3 (Claude CLI integration in-app) — backend ready

`ClaudeCliClient` complete and tested in isolation.  Wiring into
`AppState` and the agent-card UI is pending.
